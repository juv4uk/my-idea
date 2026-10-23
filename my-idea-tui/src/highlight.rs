//! Підсвічування редактора — **display-only підказка хоста**, як і підсвічування
//! збігів пошуку. Воно НЕ є другим читачем і нічого не вирішує про значення:
//! жоден токен звідси не потрапляє в обчислення, а невідома форма просто
//! лишається без кольору. Значення належить SENS; тут — лише колір рядка.
//!
//! Кольори беруться з ролей теми, тож світла/темна тема працює без окремих
//! правил підсвічування.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Comment,
    Str,
    Number,
    Paren,
    Symbol,
    Space,
}

/// Розкласти рядок на відрізки (байтові зсуви в межах рядка) з видами.
/// Розбір коректний для UTF-8: рухаємось по межах символів.
pub fn line_tokens(line: &str) -> Vec<(usize, usize, Token)> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let end_of = |index: usize| chars.get(index).map(|(offset, _)| *offset).unwrap_or(line.len());
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let (offset, ch) = chars[index];
        match ch {
            // ';' коментує до кінця рядка.
            ';' => {
                out.push((offset, line.len(), Token::Comment));
                break;
            }
            // Рядок у лапках із екрануванням «\»; незакритий тягнеться до кінця.
            '"' => {
                let mut cursor = index + 1;
                while cursor < chars.len() {
                    match chars[cursor].1 {
                        '\\' => cursor += 2,
                        '"' => {
                            cursor += 1;
                            break;
                        }
                        _ => cursor += 1,
                    }
                }
                let end = if cursor >= chars.len() { line.len() } else { end_of(cursor) };
                out.push((offset, end, Token::Str));
                index = cursor;
            }
            '(' | ')' | '[' | ']' => {
                out.push((offset, end_of(index + 1), Token::Paren));
                index += 1;
            }
            ch if ch.is_whitespace() => {
                let mut cursor = index + 1;
                while cursor < chars.len() && chars[cursor].1.is_whitespace() {
                    cursor += 1;
                }
                out.push((offset, end_of(cursor), Token::Space));
                index = cursor;
            }
            ch => {
                let numeric = ch.is_ascii_digit()
                    || ((ch == '-' || ch == '+')
                        && chars
                            .get(index + 1)
                            .map(|(_, next)| next.is_ascii_digit())
                            .unwrap_or(false));
                let mut cursor = index + 1;
                while cursor < chars.len() {
                    let next = chars[cursor].1;
                    if next.is_whitespace() || matches!(next, '(' | ')' | '[' | ']' | '"' | ';') {
                        break;
                    }
                    cursor += 1;
                }
                out.push((
                    offset,
                    end_of(cursor),
                    if numeric { Token::Number } else { Token::Symbol },
                ));
                index = cursor;
            }
        }
    }
    out
}

pub fn token_style(token: Token, theme: Theme) -> Style {
    match token {
        Token::Comment => Style::default().fg(theme.inactive()),
        Token::Str => Style::default().fg(theme.ok()),
        Token::Number => Style::default().fg(theme.graph()),
        Token::Paren => Style::default().fg(theme.inactive()),
        Token::Symbol | Token::Space => Style::default(),
    }
}

/// Рядки редактора з підсвічуванням і маркером курсора (`▏` акцентом теми).
pub fn editor_lines(buffer: &str, cursor: usize, theme: Theme) -> Vec<Line<'static>> {
    let cursor = cursor.min(buffer.len());
    let cursor_line = buffer[..cursor].bytes().filter(|byte| *byte == b'\n').count();
    let cursor_column = cursor - buffer[..cursor].rfind('\n').map(|index| index + 1).unwrap_or(0);

    buffer
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            let tokens = line_tokens(line);
            let mut spans: Vec<Span<'static>> = Vec::new();
            // Маркер курсора має стояти рівно один раз: на межі відрізків його
            // підходить і кінець попереднього, і початок наступного.
            let mut marked = false;
            for (start, end, token) in tokens {
                let style = token_style(token, theme);
                if !marked && index == cursor_line && cursor_column >= start && cursor_column <= end {
                    // Розділити відрізок навколо курсора й вставити маркер.
                    let cut = cursor_column.clamp(start, end);
                    if cut > start {
                        spans.push(Span::styled(line[start..cut].to_string(), style));
                    }
                    spans.push(Span::styled("▏".to_string(), Style::default().fg(theme.accent())));
                    if end > cut {
                        spans.push(Span::styled(line[cut..end].to_string(), style));
                    }
                    marked = true;
                    continue;
                }
                spans.push(Span::styled(line[start..end].to_string(), style));
            }
            if index == cursor_line && spans.is_empty() {
                spans.push(Span::styled("▏".to_string(), Style::default().fg(theme.accent())));
            }
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{editor_lines, line_tokens, Token};
    use crate::theme::Theme;

    fn kinds(line: &str) -> Vec<Token> {
        line_tokens(line).into_iter().map(|(_, _, kind)| kind).collect()
    }

    #[test]
    fn comments_strings_numbers_and_parens_are_distinguished() {
        assert_eq!(kinds("; це коментар"), vec![Token::Comment]);
        assert_eq!(kinds("(додати 2 3)"), vec![
            Token::Paren, Token::Symbol, Token::Space, Token::Number, Token::Space,
            Token::Number, Token::Paren,
        ]);
        // Екрановані лапки всередині рядка не закривають його.
        let tokens = line_tokens(r#"(друк "а\"б" 5)"#);
        let strings: Vec<_> = tokens.iter().filter(|(_, _, k)| *k == Token::Str).collect();
        assert_eq!(strings.len(), 1);
        let (start, end, _) = *strings[0];
        assert_eq!(&r#"(друк "а\"б" 5)"#[start..end], r#""а\"б""#);
    }

    #[test]
    fn cyrillic_symbols_do_not_split_across_utf8_boundaries() {
        let tokens = line_tokens("(площа круга)");
        for (start, end, _) in &tokens {
            assert!("(площа круга)".is_char_boundary(*start), "start {start}");
            assert!("(площа круга)".is_char_boundary(*end), "end {end}");
        }
        assert!(kinds("(площа круга)").contains(&Token::Symbol));
    }

    #[test]
    fn editor_lines_marks_the_cursor_line_without_losing_text() {
        let buffer = "(додати 2 3)\n; коментар\n";
        // Байтовий курсор після «додати»: «(» + 6 кириличних символів × 2 байти = 13.
        let lines = editor_lines(buffer, 13, Theme::Dark);
        assert_eq!(lines.len(), 3);
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(first, "(додати▏ 2 3)");
        let second: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(second, "; коментар");
    }
}
