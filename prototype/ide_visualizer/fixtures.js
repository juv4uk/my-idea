/**
 * Canonical Derivation IR Fixtures for Panini Grammar Machine
 * Format: panini-derivation-ir/0.1
 */

export const DERIVATION_FIXTURES = {
  bhavati: {
    "ir_version": "panini-derivation-ir/0.1",
    "derivation_id": "drv:canonical:bhavati-v0.1",
    "target_word": "भवति (bhavati)",
    "description": "Derivation of root √bhū (भू सत्तायाम्, 1st gaṇa Bhvādi) in present tense 3rd person singular (laṭ, tiṅ-tip).",
    "status": "success",
    "final_surface_form": "Bavati",
    "rules": [
      {
        "sutra_id": "3.2.123",
        "text_deva": "वर्तमाने लट्",
        "text_slp1": "vartamAne laT",
        "classification": "VIDHI",
        "summary": "Affixes the lakāra 'laṭ' to denote action in the present tense."
      },
      {
        "sutra_id": "3.4.78",
        "text_deva": "तिप्तस्झिसिप्थस्थमिब्वस्मस्तातांझथांसाथांध्वमिड्वहिमहिङ्",
        "text_slp1": "tiptasjhi...",
        "classification": "VIDHI",
        "summary": "Substitutes lakāra with 18 tiṅ affixes; selects 3rd singular parasmaipada 'tip'."
      },
      {
        "sutra_id": "1.3.9",
        "text_deva": "तस्य लोपः",
        "text_slp1": "tasya lopaH",
        "classification": "VIDHI",
        "summary": "Elides the it-marker 'p' (by 1.3.3 halantyam) while preserving the 'pit' property tag."
      },
      {
        "sutra_id": "3.1.68",
        "text_deva": "कर्तरि शप्",
        "text_slp1": "kartari Sap",
        "classification": "VIDHI",
        "summary": "Inserts vikaraṇa affix 'Śap' after root √bhū before sārvadhātuka affix in kartari prayoga."
      },
      {
        "sutra_id": "3.4.113",
        "text_deva": "तिङ्शित्सार्वधातुकम्",
        "text_slp1": "tiNSitsArvaDAtukam",
        "classification": "SAMJNA",
        "summary": "Assigns sārvadhātuka saṃjñā to 'Śap' (Ś-it) and 'tip' (tiṅ)."
      },
      {
        "sutra_id": "7.3.84",
        "text_deva": "सार्वधातुकार्धधातुकयोः",
        "text_slp1": "sArvaDAtukArDaDAtukayoH",
        "classification": "VIDHI",
        "summary": "Applies guṇa substitution to the final vowel of the aṅga (ū -> o) before sārvadhātuka 'a'."
      },
      {
        "sutra_id": "6.1.78",
        "text_deva": "एचोऽयवायावः",
        "text_slp1": "eco 'yavAyAvaH",
        "classification": "VIDHI",
        "summary": "Sandhi replacement: 'o' followed by vowel 'a' transforms to 'av' (bho + a -> bhav + a)."
      },
      {
        "sutra_id": "1.4.14",
        "text_deva": "सुप्तिङन्तं पदम्",
        "text_slp1": "suptiGantaM padam",
        "classification": "SAMJNA",
        "summary": "Designates the complete tiṅ-inflected form 'bhavati' as a valid syntactic Pada."
      }
    ],
    "states": [
      {
        "id": "state:bhavati:00-input",
        "hash": "state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga", "BvAdi"]
          }
        ],
        "relations": []
      },
      {
        "id": "state:bhavati:01-lat",
        "hash": "state:sha256:a6b610c14b7e8d75cf701be5e921d01ee3fdf9bcf693e506990d0b04c86ec596",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:lakara-laT",
            "kind": "lakara",
            "source_form": "laT",
            "surface_form": "laT",
            "designations": ["laT", "Tit", "vartamAna"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-BU",
            "to": "term:lakara-laT"
          }
        ]
      },
      {
        "id": "state:bhavati:02-tip",
        "hash": "state:sha256:cbb6508933b93475294025b6a7ca98516d418706bf9fc8ef21d8b9d3fc546dd4",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-tip",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "tip",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka", "parasmaipada", "praTama-puruza", "ekavacana"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-BU",
            "to": "term:tin-tip"
          }
        ]
      },
      {
        "id": "state:bhavati:03-ti",
        "hash": "state:sha256:063d8d672cf3c9ce05bb7eeebcba09e13a48e77aee2f2eeae85d8d85f867fe08",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-BU",
            "to": "term:tin-ti"
          }
        ]
      },
      {
        "id": "state:bhavati:04-sap-ti",
        "hash": "state:sha256:f1262d1645e5461c3600f682f6f387db29dbef01d848773950b7dc3f84dc248c",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:vikarana-Sap",
            "kind": "pratyaya",
            "source_form": "Sap",
            "surface_form": "Sap",
            "designations": ["vikaraRa", "pratyaya", "Sit", "pit", "sArvaDAtuka"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:root-BU",
            "to": "term:vikarana-Sap"
          },
          {
            "kind": "attachment",
            "from": "term:vikarana-Sap",
            "to": "term:tin-ti"
          }
        ]
      },
      {
        "id": "state:bhavati:05-sap-a-ti",
        "hash": "state:sha256:4aebf4e1f7fb1b608bf327b8aa0bc0646c050228aebe036573c2ee483c66f7f6",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-BU",
            "kind": "dhAtu",
            "source_form": "BU",
            "surface_form": "BU",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:vikarana-a",
            "kind": "pratyaya",
            "source_form": "Sap",
            "surface_form": "a",
            "designations": ["vikaraRa", "pratyaya", "Sit-derived", "sArvaDAtuka"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:root-BU",
            "to": "term:vikarana-a"
          },
          {
            "kind": "attachment",
            "from": "term:vikarana-a",
            "to": "term:tin-ti"
          }
        ]
      },
      {
        "id": "state:bhavati:06-guna-bo-a-ti",
        "hash": "state:sha256:d8bcaae1162d0577bf64e9a6671ca60775d79faaa498522646c6505e3ba0cbb5",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-Bo",
            "kind": "dhAtu-guna",
            "source_form": "BU",
            "surface_form": "Bo",
            "designations": ["dhAtu", "aGga", "guRa-applied"]
          },
          {
            "id": "term:vikarana-a",
            "kind": "pratyaya",
            "source_form": "Sap",
            "surface_form": "a",
            "designations": ["vikaraRa", "pratyaya", "sArvaDAtuka"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:root-Bo",
            "to": "term:vikarana-a"
          },
          {
            "kind": "attachment",
            "from": "term:vikarana-a",
            "to": "term:tin-ti"
          }
        ]
      },
      {
        "id": "state:bhavati:07-sandhi-bavati",
        "hash": "state:sha256:4d603a11b6ff03ea562ef6c41b8a531e07b8b40aa9a7852c00224d4e330ad3c3",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-Bav",
            "kind": "dhAtu-sandhi",
            "source_form": "BU",
            "surface_form": "Bav",
            "designations": ["dhAtu", "aGga", "av-AdeSa"]
          },
          {
            "id": "term:vikarana-a",
            "kind": "pratyaya",
            "source_form": "Sap",
            "surface_form": "a",
            "designations": ["vikaraRa", "pratyaya"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya"]
          }
        ],
        "relations": [
          {
            "kind": "fusion",
            "from": "term:root-Bav",
            "to": "term:vikarana-a"
          }
        ]
      },
      {
        "id": "state:bhavati:08-pada-bavati",
        "hash": "state:sha256:6e01a88bbca5603c4f74d0a3d6a4cba8f0e5bcf27fae782d4993ea089e9be410",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:pada-Bavati",
            "kind": "pada",
            "source_form": "BU+laT",
            "surface_form": "Bavati",
            "designations": ["tiGanta-pada", "prathama-purusha", "ekavacana"]
          }
        ],
        "relations": []
      }
    ],
    "events": [
      {
        "event_id": "evt:01",
        "kind": "state-observed",
        "depends_on": [],
        "payload": {
          "state": "state:bhavati:00-input",
          "hash": "state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7"
        }
      },
      {
        "event_id": "evt:02",
        "kind": "applicability-check",
        "depends_on": ["evt:01"],
        "payload": {
          "rule": "3.2.123",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:03",
        "kind": "rule-decision",
        "depends_on": ["evt:02"],
        "payload": {
          "rule": "3.2.123",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:04",
        "kind": "state-transition",
        "depends_on": ["evt:03"],
        "payload": {
          "rule": "3.2.123",
          "before": "state:bhavati:00-input",
          "after": "state:bhavati:01-lat",
          "operation": "attach-lakara-laT"
        }
      },
      {
        "event_id": "evt:05",
        "kind": "applicability-check",
        "depends_on": ["evt:04"],
        "payload": {
          "rule": "3.4.78",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:06",
        "kind": "rule-decision",
        "depends_on": ["evt:05"],
        "payload": {
          "rule": "3.4.78",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:07",
        "kind": "state-transition",
        "depends_on": ["evt:06"],
        "payload": {
          "rule": "3.4.78",
          "before": "state:bhavati:01-lat",
          "after": "state:bhavati:02-tip",
          "operation": "select-tin-tip"
        }
      },
      {
        "event_id": "evt:08",
        "kind": "applicability-check",
        "depends_on": ["evt:07"],
        "payload": {
          "rule": "1.3.9",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:09",
        "kind": "rule-decision",
        "depends_on": ["evt:08"],
        "payload": {
          "rule": "1.3.9",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:10",
        "kind": "state-transition",
        "depends_on": ["evt:09"],
        "payload": {
          "rule": "1.3.9",
          "before": "state:bhavati:02-tip",
          "after": "state:bhavati:03-ti",
          "operation": "elide-it-p-preserve-pit"
        }
      },
      {
        "event_id": "evt:11",
        "kind": "applicability-check",
        "depends_on": ["evt:10"],
        "payload": {
          "rule": "3.1.68",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:12",
        "kind": "rule-decision",
        "depends_on": ["evt:11"],
        "payload": {
          "rule": "3.1.68",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:13",
        "kind": "state-transition",
        "depends_on": ["evt:12"],
        "payload": {
          "rule": "3.1.68",
          "before": "state:bhavati:03-ti",
          "after": "state:bhavati:04-sap-ti",
          "operation": "insert-vikarana-Sap"
        }
      },
      {
        "event_id": "evt:14",
        "kind": "applicability-check",
        "depends_on": ["evt:13"],
        "payload": {
          "rule": "3.4.113",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:15",
        "kind": "rule-decision",
        "depends_on": ["evt:14"],
        "payload": {
          "rule": "3.4.113",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:16",
        "kind": "state-transition",
        "depends_on": ["evt:15"],
        "payload": {
          "rule": "3.4.113",
          "before": "state:bhavati:04-sap-ti",
          "after": "state:bhavati:05-sap-a-ti",
          "operation": "elide-S-p-and-designate-sarvadhatuka"
        }
      },
      {
        "event_id": "evt:17",
        "kind": "applicability-check",
        "depends_on": ["evt:16"],
        "payload": {
          "rule": "7.3.84",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:18",
        "kind": "rule-decision",
        "depends_on": ["evt:17"],
        "payload": {
          "rule": "7.3.84",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:19",
        "kind": "state-transition",
        "depends_on": ["evt:18"],
        "payload": {
          "rule": "7.3.84",
          "before": "state:bhavati:05-sap-a-ti",
          "after": "state:bhavati:06-guna-bo-a-ti",
          "operation": "apply-guna-U-to-o"
        }
      },
      {
        "event_id": "evt:20",
        "kind": "applicability-check",
        "depends_on": ["evt:19"],
        "payload": {
          "rule": "6.1.78",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:21",
        "kind": "rule-decision",
        "depends_on": ["evt:20"],
        "payload": {
          "rule": "6.1.78",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:22",
        "kind": "state-transition",
        "depends_on": ["evt:21"],
        "payload": {
          "rule": "6.1.78",
          "before": "state:bhavati:06-guna-bo-a-ti",
          "after": "state:bhavati:07-sandhi-bavati",
          "operation": "apply-eco-sandhi-o-a-to-av-a"
        }
      },
      {
        "event_id": "evt:23",
        "kind": "applicability-check",
        "depends_on": ["evt:22"],
        "payload": {
          "rule": "1.4.14",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:24",
        "kind": "rule-decision",
        "depends_on": ["evt:23"],
        "payload": {
          "rule": "1.4.14",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:25",
        "kind": "state-transition",
        "depends_on": ["evt:24"],
        "payload": {
          "rule": "1.4.14",
          "before": "state:bhavati:07-sandhi-bavati",
          "after": "state:bhavati:08-pada-bavati",
          "operation": "assign-pada-samjna"
        }
      },
      {
        "event_id": "evt:26",
        "kind": "trace-terminated",
        "depends_on": ["evt:25"],
        "payload": {
          "outcome": "success",
          "final_state": "state:bhavati:08-pada-bavati"
        }
      }
    ]
  },

  dadati: {
    "ir_version": "panini-derivation-ir/0.1",
    "derivation_id": "drv:canonical:dadati-v0.1",
    "target_word": "ददाति (dadāti)",
    "description": "Derivation of root √dā (डुदाञ् दाने, 3rd gaṇa Juhotyādi) demonstrating Apavāda rule selection (2.4.75 ślu blocking 3.1.68 śap), dvirvacana (6.1.10), hrasva (7.4.59), and non-ik guṇa prohibition (1.1.3).",
    "status": "success",
    "final_surface_form": "dadAti",
    "rules": [
      {
        "sutra_id": "3.2.123",
        "text_deva": "वर्तमाने लट्",
        "text_slp1": "vartamAne laT",
        "classification": "VIDHI",
        "summary": "Affixes lakāra 'laṭ' in the present tense."
      },
      {
        "sutra_id": "3.4.78",
        "text_deva": "तिप्तस्झि...",
        "text_slp1": "tiptasjhi...",
        "classification": "VIDHI",
        "summary": "Substitutes lakāra with parasmaipada singular 'tip'."
      },
      {
        "sutra_id": "1.3.9",
        "text_deva": "तस्य लोपः",
        "text_slp1": "tasya lopaH",
        "classification": "VIDHI",
        "summary": "Elides it-letter 'p' from 'tip'."
      },
      {
        "sutra_id": "3.1.68",
        "text_deva": "कर्तरि शप्",
        "text_slp1": "kartari Sap",
        "classification": "VIDHI",
        "summary": "General utsarga rule: inserts vikaraṇa Śap."
      },
      {
        "sutra_id": "2.4.75",
        "text_deva": "जुहोत्यादिभ्यः श्लुः",
        "text_slp1": "juhotyAdibhyaH SluH",
        "classification": "VIDHI",
        "is_apavada_for": "3.1.68",
        "summary": "Special apavāda rule: replaces Śap with Ślu (zero-affix) for Juhotyādi class roots."
      },
      {
        "sutra_id": "6.1.10",
        "text_deva": "श्लौ",
        "text_slp1": "SlO",
        "classification": "VIDHI",
        "summary": "Triggers reduplication (dvirvacana) of the root when followed by Ślu."
      },
      {
        "sutra_id": "6.1.4",
        "text_deva": "पूर्वोऽभ्यासः",
        "text_slp1": "pUrvo'BhyAsaH",
        "classification": "SAMJNA",
        "summary": "Assigns abhyāsa saṃjñā to the first (prior) reduplicated syllable."
      },
      {
        "sutra_id": "7.4.59",
        "text_deva": "ह्रस्वः",
        "text_slp1": "hrasvaH",
        "classification": "VIDHI",
        "summary": "Shortens the vowel of the abhyāsa syllable (dā -> da)."
      },
      {
        "sutra_id": "1.1.3",
        "text_deva": "इको गुणवृद्धी",
        "text_slp1": "iko guRavfdDI",
        "classification": "PARIBHASA",
        "summary": "Guṇa and vṛddhi substitutions apply only to 'ik' vowels (i, u, ṛ, ḷ); prohibits guṇa for root vowel 'ā'."
      },
      {
        "sutra_id": "1.4.14",
        "text_deva": "सुप्तिङन्तं पदम्",
        "text_slp1": "suptiGantaM padam",
        "classification": "SAMJNA",
        "summary": "Assigns pada saṃjñā to 'dadāti'."
      }
    ],
    "states": [
      {
        "id": "state:dadati:00-input",
        "hash": "state:sha256:c0e648be78f3d1b711bc9cb04bc86efbb9ee21ffb3ae19fa82ee06e408db06d6",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga", "juhotyAdi"]
          }
        ],
        "relations": []
      },
      {
        "id": "state:dadati:01-lat",
        "hash": "state:sha256:7b50ef4a65490cf8519ca42ee5fb4f97184ef4fa68297b4169ec069f06be368a",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:lakara-laT",
            "kind": "lakara",
            "source_form": "laT",
            "surface_form": "laT",
            "designations": ["laT", "Tit"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-dA",
            "to": "term:lakara-laT"
          }
        ]
      },
      {
        "id": "state:dadati:02-tip",
        "hash": "state:sha256:0d5ea66f7f6a7d57be415f33f95b8719277f98eece47b4df44c5ecdae4cbb555",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-tip",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "tip",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-dA",
            "to": "term:tin-tip"
          }
        ]
      },
      {
        "id": "state:dadati:03-ti",
        "hash": "state:sha256:88aeb78a1c87515eeea2a0df8b8358e69ee14eb5e76a6cfab74d75d6fa7c244b",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit", "sArvaDAtuka"]
          }
        ],
        "relations": [
          {
            "kind": "attachment",
            "from": "term:root-dA",
            "to": "term:tin-ti"
          }
        ]
      },
      {
        "id": "state:dadati:04-sap-ti",
        "hash": "state:sha256:0d6ce74a3f4e24ef547513fe9a80e12d46e2fc73295ce8129df4f713e8b4e768",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:vikarana-Sap",
            "kind": "pratyaya",
            "source_form": "Sap",
            "surface_form": "Sap",
            "designations": ["vikaraRa", "pratyaya"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:root-dA",
            "to": "term:vikarana-Sap"
          }
        ]
      },
      {
        "id": "state:dadati:05-slu-ti",
        "hash": "state:sha256:3eb761a2da387f374ea86dc51ef1c28c89791005a9eead4e0e5a95cb88421884",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:marker-Slu",
            "kind": "lopa-marker",
            "source_form": "Slu",
            "surface_form": "",
            "designations": ["Slu", "lopa", "dvirvacana-trigger"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya", "pit"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:root-dA",
            "to": "term:marker-Slu"
          }
        ]
      },
      {
        "id": "state:dadati:06-dvirvacana",
        "hash": "state:sha256:e0b656b27d49ba8e9c704e6c96a84f3da6a7dbfa6ecceb0d39e31d4eebe76595",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:abhyasa-dA",
            "kind": "abhyasa-dhatu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["abhyAsa", "pUrva"]
          },
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya"]
          }
        ],
        "relations": [
          {
            "kind": "reduplication",
            "from": "term:root-dA",
            "to": "term:abhyasa-dA"
          }
        ]
      },
      {
        "id": "state:dadati:07-hrasva",
        "hash": "state:sha256:69b4009772ee571d87e07cb2cfdbdaee98c76caefeaae282bdaea45a60e5aa06",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:abhyasa-da",
            "kind": "abhyasa-dhatu-hrasva",
            "source_form": "dA",
            "surface_form": "da",
            "designations": ["abhyAsa", "hrasva"]
          },
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:abhyasa-da",
            "to": "term:root-dA"
          }
        ]
      },
      {
        "id": "state:dadati:08-guna-prohibited",
        "hash": "state:sha256:69b4009772ee571d87e07cb2cfdbdaee98c76caefeaae282bdaea45a60e5aa06",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:abhyasa-da",
            "kind": "abhyasa-dhatu-hrasva",
            "source_form": "dA",
            "surface_form": "da",
            "designations": ["abhyAsa", "hrasva"]
          },
          {
            "id": "term:root-dA",
            "kind": "dhAtu",
            "source_form": "dA",
            "surface_form": "dA",
            "designations": ["dhAtu", "aGga"]
          },
          {
            "id": "term:tin-ti",
            "kind": "pratyaya",
            "source_form": "tip",
            "surface_form": "ti",
            "designations": ["tiN", "pratyaya"]
          }
        ],
        "relations": [
          {
            "kind": "scope",
            "from": "term:abhyasa-da",
            "to": "term:root-dA"
          }
        ]
      },
      {
        "id": "state:dadati:09-pada-dadati",
        "hash": "state:sha256:91df454559c5d0130dbb6e22f87ee261dbeab5fa2d512a2a07c08fe717ca14e0",
        "schema": "panini-state/0.1",
        "serialization": "canonical-json-sha256-v0.1",
        "terms": [
          {
            "id": "term:pada-dadAti",
            "kind": "pada",
            "source_form": "dA+laT",
            "surface_form": "dadAti",
            "designations": ["tiGanta-pada", "prathama-purusha", "ekavacana"]
          }
        ],
        "relations": []
      }
    ],
    "events": [
      {
        "event_id": "evt:01",
        "kind": "state-observed",
        "depends_on": [],
        "payload": {
          "state": "state:dadati:00-input",
          "hash": "state:sha256:c0e648be78f3d1b711bc9cb04bc86efbb9ee21ffb3ae19fa82ee06e408db06d6"
        }
      },
      {
        "event_id": "evt:02",
        "kind": "applicability-check",
        "depends_on": ["evt:01"],
        "payload": {
          "rule": "3.2.123",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:03",
        "kind": "rule-decision",
        "depends_on": ["evt:02"],
        "payload": {
          "rule": "3.2.123",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:04",
        "kind": "state-transition",
        "depends_on": ["evt:03"],
        "payload": {
          "rule": "3.2.123",
          "before": "state:dadati:00-input",
          "after": "state:dadati:01-lat",
          "operation": "attach-lakara-laT"
        }
      },
      {
        "event_id": "evt:05",
        "kind": "applicability-check",
        "depends_on": ["evt:04"],
        "payload": {
          "rule": "3.4.78",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:06",
        "kind": "rule-decision",
        "depends_on": ["evt:05"],
        "payload": {
          "rule": "3.4.78",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:07",
        "kind": "state-transition",
        "depends_on": ["evt:06"],
        "payload": {
          "rule": "3.4.78",
          "before": "state:dadati:01-lat",
          "after": "state:dadati:02-tip",
          "operation": "select-tin-tip"
        }
      },
      {
        "event_id": "evt:08",
        "kind": "applicability-check",
        "depends_on": ["evt:07"],
        "payload": {
          "rule": "1.3.9",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:09",
        "kind": "rule-decision",
        "depends_on": ["evt:08"],
        "payload": {
          "rule": "1.3.9",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:10",
        "kind": "state-transition",
        "depends_on": ["evt:09"],
        "payload": {
          "rule": "1.3.9",
          "before": "state:dadati:02-tip",
          "after": "state:dadati:03-ti",
          "operation": "elide-it-p"
        }
      },
      {
        "event_id": "evt:11",
        "kind": "applicability-check",
        "depends_on": ["evt:10"],
        "payload": {
          "rule": "3.1.68",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:12",
        "kind": "rule-decision",
        "depends_on": ["evt:11"],
        "payload": {
          "rule": "2.4.75",
          "decision": "apavada-over-utsarga",
          "overridden_rule": "3.1.68"
        }
      },
      {
        "event_id": "evt:13",
        "kind": "state-transition",
        "depends_on": ["evt:12"],
        "payload": {
          "rule": "2.4.75",
          "before": "state:dadati:03-ti",
          "after": "state:dadati:05-slu-ti",
          "operation": "replace-Sap-with-Slu"
        }
      },
      {
        "event_id": "evt:14",
        "kind": "applicability-check",
        "depends_on": ["evt:13"],
        "payload": {
          "rule": "6.1.10",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:15",
        "kind": "rule-decision",
        "depends_on": ["evt:14"],
        "payload": {
          "rule": "6.1.10",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:16",
        "kind": "state-transition",
        "depends_on": ["evt:15"],
        "payload": {
          "rule": "6.1.10",
          "before": "state:dadati:05-slu-ti",
          "after": "state:dadati:06-dvirvacana",
          "operation": "reduplicate-root-dA"
        }
      },
      {
        "event_id": "evt:17",
        "kind": "applicability-check",
        "depends_on": ["evt:16"],
        "payload": {
          "rule": "7.4.59",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:18",
        "kind": "rule-decision",
        "depends_on": ["evt:17"],
        "payload": {
          "rule": "7.4.59",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:19",
        "kind": "state-transition",
        "depends_on": ["evt:18"],
        "payload": {
          "rule": "7.4.59",
          "before": "state:dadati:06-dvirvacana",
          "after": "state:dadati:07-hrasva",
          "operation": "shorten-abhyasa-vowel-A-to-a"
        }
      },
      {
        "event_id": "evt:20",
        "kind": "applicability-check",
        "depends_on": ["evt:19"],
        "payload": {
          "rule": "1.1.3",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:21",
        "kind": "rule-decision",
        "depends_on": ["evt:20"],
        "payload": {
          "rule": "1.1.3",
          "decision": "guna-prohibited-non-ik"
        }
      },
      {
        "event_id": "evt:22",
        "kind": "state-transition",
        "depends_on": ["evt:21"],
        "payload": {
          "rule": "1.1.3",
          "before": "state:dadati:07-hrasva",
          "after": "state:dadati:08-guna-prohibited",
          "operation": "retain-surface-root-form"
        }
      },
      {
        "event_id": "evt:23",
        "kind": "applicability-check",
        "depends_on": ["evt:22"],
        "payload": {
          "rule": "1.4.14",
          "outcome": "applicable"
        }
      },
      {
        "event_id": "evt:24",
        "kind": "rule-decision",
        "depends_on": ["evt:23"],
        "payload": {
          "rule": "1.4.14",
          "decision": "selected"
        }
      },
      {
        "event_id": "evt:25",
        "kind": "state-transition",
        "depends_on": ["evt:24"],
        "payload": {
          "rule": "1.4.14",
          "before": "state:dadati:08-guna-prohibited",
          "after": "state:dadati:09-pada-dadati",
          "operation": "assign-pada-samjna"
        }
      },
      {
        "event_id": "evt:26",
        "kind": "trace-terminated",
        "depends_on": ["evt:25"],
        "payload": {
          "outcome": "success",
          "final_state": "state:dadati:09-pada-dadati"
        }
      }
    ]
  }
};
