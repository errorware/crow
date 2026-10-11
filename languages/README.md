# Language packs

Crow's screens come from messages: `assets/i18n/en.ftl` holds every one in
English, built into Crow. A language pack is the same messages in another
language, in one [Fluent](https://projectfluent.org/fluent/guide/) file.

| Pack | Language | Coverage |
|---|---|---|
| [`es.ftl`](es.ftl) | Español (Spanish) | complete |
| [`ja.ftl`](ja.ftl) | 日本語 (Japanese) | complete |

## Install one

1. Copy the `.ftl` file into `~/.config/crow/languages/`
   (Settings → Personalisation → Language → OPEN LANGUAGES FOLDER).
2. Click RELOAD, then pick it. Crow switches at once; no restart.

## Make one

1. Settings → Personalisation → Language → EXPORT ENGLISH TEMPLATE writes
   every English message into the languages folder as `template.ftl.txt`.
2. Rename it to your language's code: `de.ftl`, `pt-BR.ftl`, `zh-Hans.ftl`.
3. Fill in the header at the top:

   ```
   ### name: German
   ### native: Deutsch
   ### author: your name
   ### crow: 0.8.2
   ### font: (only if JetBrains Mono lacks your script, e.g. Noto Sans CJK JP)
   ```

4. Translate the text after each `=`. Leave the ids before it alone.
5. Check it: `crow --check-language de.ftl` prints the coverage and every
   message it refused; add `--missing` to list what's left to translate.

You don't have to translate everything: whatever a pack lacks shows in
English. Translating in pieces is fine.

## The rules

A pack's text is shown as Crow's own, so Crow holds each message to the
English one. A message that breaks a rule is refused and shows in English
(Settings lists them):

- **The same placeholders as English.** `{ $server }` in English must be in
  the translation, and nothing English doesn't have may be added. This is
  what stops a pack from dropping the server's name out of a confirmation.
- **No control or direction-override characters** (U+202A–202E,
  U+2066–2069 and the like), which can make text read differently from how
  it is stored.
- **No pulling in other messages** (`{ other-message }`): each message is
  checked on its own.
- **Text only.** There's no markup, no links, nothing that runs.

Danger prompts (Danger Zone, fleet runs, host-key changes) also show the
English under a translation, whatever a pack says.

## Plurals and choices

Counts are numbers, so a message can choose its words by your language's
plural rules:

```
fleet-overview-checking-2 = Checking { $n } { $n ->
        [one] server
       *[other] servers
    }
```

Your language may need other categories (`few`, `many`, …) or only `other`
(Japanese); the `*` marks the default. A few messages choose on a word Crow
passes instead (`[yes]`/`[no]`, `[security]`/`[kernel]`); keep those keys.

## What stays English

Commands Crow runs, config file contents and directive names
(`PermitRootLogin`), typed confirmation keywords (`ROTATE`, `CONFIRM`), the
audit log's stored records, and exported reports (posture, DR plan, incident
timeline), which are often read by someone else. None of those are in
`en.ftl`.

## Testing a layout

`CROW_I18N_PSEUDO=1 crow` adds a pseudo-language to the picker: English with
accents and a third more length, so any text that isn't a message (plain
English) and any layout that can't take a longer translation stand out.
