//! Мінімальний Editor API для my-lisp-плагінів (issue #9, IDE-LISP-API-1),
//! розширений хуками/keymaps (issue #11, IDE-LISP-HOOKS-1). Хост
//! (Rust/Tauri) надає лише механізм — реєстрацію команд, повідомлення,
//! текст буфера, виділення, заміну виділення, прив'язку клавіш і підписку
//! на події. Поведінка (що саме робить команда чи обробник) лишається
//! Lisp-owned. Семантику Lisp тут не дублюємо: неавторизовані
//! host-можливості (файли, процеси) лишаються недоступними, бо жодна
//! `read-file`-подібна capability тут не реєструється.
//!
//! Minimal Editor API for my-lisp plugins (issue #9, IDE-LISP-API-1),
//! extended with hooks/keymaps (issue #11, IDE-LISP-HOOKS-1). The host
//! (Rust/Tauri) exposes mechanism only — command registration, message,
//! buffer text, selection, replace-selection, key binding, event
//! subscription — never duplicates Lisp semantics. Unauthorized host
//! capabilities (filesystem, processes) stay unreachable because no
//! `read-file`-like capability is registered here.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

use my_lisp::{
    eval_expr, exact_arity, register_capability, Environment, ErrorKind, Expr, LanguageError,
    Span, Value,
};

use crate::ReplSession;

const TOKEN_ENV_KEY: &str = "%editor-registry-token%";
const INVOKE_TARGET_KEY: &str = "editor-internal-invoke-target";
const HANDLE_KIND: &str = "editor-registry";

/// Одна команда, зареєстрована з Lisp-плагіна.
/// One command registered from a Lisp plugin.
struct RegisteredCommand {
    name: String,
    handler: Value,
}

/// Прив'язка клавіші до імені вже (чи ще не) зареєстрованої команди
/// (issue #11). Валідність імені команди перевіряється при диспетчеризації
/// клавіші, не при біндингу — порядок завантаження плагінів не фіксований
/// щодо того, яка команда/keymap приходить першою.
///
/// A key bound to a command name (issue #11). The command name's validity
/// is checked at key-dispatch time, not at bind time — plugin load order
/// doesn't guarantee a command is registered before its keymap is.
struct RegisteredKeymap {
    key: String,
    command: String,
}

/// Один обробник однієї події редактора, зареєстрований під власним
/// `handler_id` — той самий `retain`-потім-`push` патерн, що й у команд,
/// щоб reload того самого плагіна замінював обробник, а не дублював його.
///
/// One handler for one editor event, registered under its own
/// `handler_id` — the same retain-then-push pattern as commands, so
/// reloading the same plugin replaces the handler instead of duplicating it.
struct RegisteredHandler {
    event: String,
    handler_id: String,
    callback: Value,
}

#[derive(Default)]
struct RegistryState {
    commands: Vec<RegisteredCommand>,
    keymaps: Vec<RegisteredKeymap>,
    handlers: Vec<RegisteredHandler>,
    current: Option<EditorState>,
    effect: EditorEffect,
}

// Стан реєстрів — per-thread, а не process-global: `my_lisp::Session` є
// `!Send` і завжди живе на одному потоці (див. `ManagedReplSession`), а
// сама capability-функція (`HostFn`) — звичайний `fn`-покажчик без
// замикання, тож стан мусить діставатись через `Environment`, а не з
// captured state. `thread_local!` ізолює тести/сесії одна від одної.
//
// Registry state is per-thread, not process-global: `my_lisp::Session` is
// `!Send` and always lives on one thread (see `ManagedReplSession`), and
// the capability function itself (`HostFn`) is a plain `fn` pointer with
// no closure, so state must be reached through the `Environment`, not
// captured state. `thread_local!` keeps tests/sessions from colliding.
thread_local! {
    static REGISTRIES: RefCell<HashMap<u64, Rc<RefCell<RegistryState>>>> = RefCell::new(HashMap::new());
    static NEXT_TOKEN: Cell<u64> = Cell::new(1);
}

fn state_for(token: u64) -> Rc<RefCell<RegistryState>> {
    REGISTRIES.with(|registries| {
        registries
            .borrow()
            .get(&token)
            .cloned()
            .expect("editor registry token must be registered before use")
    })
}

fn token_from_environment(environment: &Environment, span: Span) -> Result<u64, LanguageError> {
    match environment.get(TOKEN_ENV_KEY).as_ref() {
        Some(Value::HostHandle { kind, token }) if &**kind == HANDLE_KIND => Ok(*token),
        _ => Err(LanguageError::new(
            ErrorKind::UnknownSymbol,
            "editor API used outside an EditorCommandRegistry-installed session",
            span,
        )),
    }
}

fn expect_string(value: &Value, what: &str, span: Span) -> Result<Rc<str>, LanguageError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            format!("{what} expects a string"),
            span,
        )),
    }
}

fn editor_register_command(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/register-command", arguments, 3, span)?;
    let token = token_from_environment(environment, span)?;
    let name = expect_string(
        &eval_expr(&arguments[0], environment)?,
        "editor/register-command",
        arguments[0].span,
    )?;
    let handler = eval_expr(&arguments[1], environment)?;
    // Опис (третій аргумент) наразі лише валідується як рядок — контракт
    // тесту вимагає його передавати, але не читає назад.
    // The description (third argument) is validated as a string for now —
    // the test contract requires passing it but never reads it back.
    expect_string(
        &eval_expr(&arguments[2], environment)?,
        "editor/register-command",
        arguments[2].span,
    )?;

    let state = state_for(token);
    let mut state = state.borrow_mut();
    state.commands.retain(|existing| existing.name != *name);
    state.commands.push(RegisteredCommand {
        name: name.to_string(),
        handler,
    });
    Ok(Value::Symbol(Rc::from("ok")))
}

fn editor_selection(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/selection", arguments, 0, span)?;
    let token = token_from_environment(environment, span)?;
    let selection = state_for(token)
        .borrow()
        .current
        .as_ref()
        .map(|state| state.selection.clone())
        .unwrap_or_default();
    Ok(Value::String(Rc::from(selection.as_str())))
}

fn editor_buffer_text(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/buffer-text", arguments, 0, span)?;
    let token = token_from_environment(environment, span)?;
    let buffer = state_for(token)
        .borrow()
        .current
        .as_ref()
        .map(|state| state.buffer.clone())
        .unwrap_or_default();
    Ok(Value::String(Rc::from(buffer.as_str())))
}

fn editor_replace_selection(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/replace-selection", arguments, 1, span)?;
    let token = token_from_environment(environment, span)?;
    let text = expect_string(
        &eval_expr(&arguments[0], environment)?,
        "editor/replace-selection",
        arguments[0].span,
    )?;
    state_for(token).borrow_mut().effect.replacement = Some(text.to_string());
    Ok(Value::String(text))
}

fn editor_message(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/message", arguments, 1, span)?;
    let token = token_from_environment(environment, span)?;
    let text = expect_string(
        &eval_expr(&arguments[0], environment)?,
        "editor/message",
        arguments[0].span,
    )?;
    state_for(token).borrow_mut().effect.message = Some(text.to_string());
    Ok(Value::String(text))
}

fn editor_keymap(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/keymap", arguments, 2, span)?;
    let token = token_from_environment(environment, span)?;
    let key = expect_string(
        &eval_expr(&arguments[0], environment)?,
        "editor/keymap",
        arguments[0].span,
    )?;
    let command = expect_string(
        &eval_expr(&arguments[1], environment)?,
        "editor/keymap",
        arguments[1].span,
    )?;

    let state = state_for(token);
    let mut state = state.borrow_mut();
    state.keymaps.retain(|existing| existing.key != *key);
    state.keymaps.push(RegisteredKeymap {
        key: key.to_string(),
        command: command.to_string(),
    });
    Ok(Value::Symbol(Rc::from("ok")))
}

fn editor_on(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("editor/on", arguments, 3, span)?;
    let token = token_from_environment(environment, span)?;
    let event = expect_string(
        &eval_expr(&arguments[0], environment)?,
        "editor/on",
        arguments[0].span,
    )?;
    let handler_id = expect_string(
        &eval_expr(&arguments[1], environment)?,
        "editor/on",
        arguments[1].span,
    )?;
    let callback = eval_expr(&arguments[2], environment)?;

    let state = state_for(token);
    let mut state = state.borrow_mut();
    state
        .handlers
        .retain(|existing| !(existing.event == *event && existing.handler_id == *handler_id));
    state.handlers.push(RegisteredHandler {
        event: event.to_string(),
        handler_id: handler_id.to_string(),
        callback,
    });
    Ok(Value::Symbol(Rc::from("ok")))
}

fn ensure_capabilities_installed() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        register_capability("editor/register-command", editor_register_command);
        register_capability("editor/selection", editor_selection);
        register_capability("editor/buffer-text", editor_buffer_text);
        register_capability("editor/replace-selection", editor_replace_selection);
        register_capability("editor/message", editor_message);
        register_capability("editor/keymap", editor_keymap);
        register_capability("editor/on", editor_on);
    });
}

/// Знімок стану редактора, видимий плагіну під час виклику команди.
/// A snapshot of editor state visible to a plugin during command invocation.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorState {
    pub buffer: String,
    pub selection: String,
}

/// Побічний ефект, який команда плагіна попросила застосувати до редактора.
/// The effect a plugin command asked the editor to apply.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorEffect {
    pub replacement: Option<String>,
    pub message: Option<String>,
}

/// Реєстр команд редактора, встановлених з canonical `.lisp`-плагінів.
/// Кожен реєстр має власний непідробний токен (`Value::HostHandle`), тож
/// декілька реєстрів/сесій в одному потоці не змішують свій стан.
///
/// Registry of editor commands installed from canonical `.lisp` plugins.
/// Each registry carries its own unforgeable token (`Value::HostHandle`),
/// so multiple registries/sessions on one thread never mix state.
pub struct EditorCommandRegistry {
    token: u64,
}

impl Default for EditorCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCommandRegistry {
    pub fn new() -> Self {
        let token = NEXT_TOKEN.with(|next| {
            let value = next.get();
            next.set(value + 1);
            value
        });
        REGISTRIES.with(|registries| {
            registries
                .borrow_mut()
                .insert(token, Rc::new(RefCell::new(RegistryState::default())));
        });
        Self { token }
    }

    /// Встановлює цей реєстр у сесію REPL: активує capability-функції
    /// `editor/*` (один раз на процес) і прив'язує токен цього реєстру.
    ///
    /// Installs this registry into a REPL session: activates the
    /// `editor/*` capability functions (once per process) and binds this
    /// registry's own token into the session.
    pub fn install_into(&self, repl: &mut ReplSession) {
        ensure_capabilities_installed();
        repl.environment().define(
            TOKEN_ENV_KEY,
            Value::HostHandle {
                kind: Rc::from(HANDLE_KIND),
                token: self.token,
            },
        );
    }

    pub fn has_command(&self, name: &str) -> bool {
        state_for(self.token)
            .borrow()
            .commands
            .iter()
            .any(|command| command.name == name)
    }

    pub fn list_commands(&self) -> Vec<String> {
        state_for(self.token)
            .borrow()
            .commands
            .iter()
            .map(|command| command.name.clone())
            .collect()
    }

    /// Викликає зареєстровану команду плагіна з поточним станом редактора і
    /// повертає ефект, який вона попросила застосувати.
    ///
    /// Invokes a registered plugin command against the given editor state
    /// and returns the effect it asked to apply.
    pub fn invoke_command(
        &self,
        repl: &mut ReplSession,
        name: &str,
        state: &EditorState,
    ) -> Result<EditorEffect, String> {
        let handler = state_for(self.token)
            .borrow()
            .commands
            .iter()
            .find(|command| command.name == name)
            .map(|command| command.handler.clone())
            .ok_or_else(|| format!("unknown command: {name}"))?;

        self.call_with_state(repl, handler, state)
    }

    pub fn has_keymap(&self, key: &str) -> bool {
        state_for(self.token)
            .borrow()
            .keymaps
            .iter()
            .any(|keymap| keymap.key == key)
    }

    pub fn list_keymaps(&self) -> Vec<(String, String)> {
        state_for(self.token)
            .borrow()
            .keymaps
            .iter()
            .map(|keymap| (keymap.key.clone(), keymap.command.clone()))
            .collect()
    }

    /// Диспетчеризує натискання клавіші: невідома клавіша чи команда, на
    /// яку вона вказує, — обидві помилки-fail-closed, ніколи не
    /// мовчазний no-op (issue #11).
    ///
    /// Dispatches a key press: an unbound key or a keymap pointing at a
    /// command that doesn't exist are both fail-closed errors, never a
    /// silent no-op (issue #11).
    pub fn dispatch_key(
        &self,
        repl: &mut ReplSession,
        key: &str,
        state: &EditorState,
    ) -> Result<EditorEffect, String> {
        let command = state_for(self.token)
            .borrow()
            .keymaps
            .iter()
            .find(|keymap| keymap.key == key)
            .map(|keymap| keymap.command.clone())
            .ok_or_else(|| format!("no command bound to key: {key}"))?;

        self.invoke_command(repl, &command, state)
    }

    pub fn handler_count(&self, event: &str) -> usize {
        state_for(self.token)
            .borrow()
            .handlers
            .iter()
            .filter(|handler| handler.event == event)
            .count()
    }

    /// Викликає кожен обробник, підписаний на `event`, ізольовано: один
    /// обробник, що впав, не зупиняє решту (той самий принцип ізоляції,
    /// що й для завантаження плагінів, issue #10).
    ///
    /// Runs every handler subscribed to `event`, isolated: one handler
    /// failing does not stop the rest (the same isolation principle as
    /// plugin loading, issue #10).
    pub fn emit_event(
        &self,
        repl: &mut ReplSession,
        event: &str,
        state: &EditorState,
    ) -> Vec<Result<EditorEffect, String>> {
        let callbacks: Vec<Value> = state_for(self.token)
            .borrow()
            .handlers
            .iter()
            .filter(|handler| handler.event == event)
            .map(|handler| handler.callback.clone())
            .collect();

        callbacks
            .into_iter()
            .map(|callback| self.call_with_state(repl, callback, state))
            .collect()
    }

    /// Викликає збережене замикання (команда чи обробник події) через
    /// тимчасову дочірню область — жодного приватного `apply`-API
    /// `my-lisp` не потребує, лише публічні
    /// `parse`/`eval_expr`/`Environment::define`.
    ///
    /// Invokes a stored closure (command or event handler) through a
    /// throwaway child scope -- this needs none of `my-lisp`'s private
    /// `apply` API, only the public `parse`/`eval_expr`/`Environment::define`.
    fn call_with_state(
        &self,
        repl: &mut ReplSession,
        callback: Value,
        state: &EditorState,
    ) -> Result<EditorEffect, String> {
        {
            let registry_state = state_for(self.token);
            let mut registry_state = registry_state.borrow_mut();
            registry_state.current = Some(state.clone());
            registry_state.effect = EditorEffect::default();
        }

        let call_environment = repl.environment().child();
        call_environment.define(INVOKE_TARGET_KEY, callback);
        let call_expr = my_lisp::parse(&format!("({INVOKE_TARGET_KEY})"))
            .map_err(|error| error.to_string())?
            .into_iter()
            .next()
            .ok_or_else(|| "editor callback invocation produced no expression".to_string())?;
        eval_expr(&call_expr, &call_environment).map_err(|error| error.to_string())?;

        let effect = state_for(self.token).borrow().effect.clone();
        Ok(effect)
    }
}
