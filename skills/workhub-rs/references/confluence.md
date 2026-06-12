# Confluence Commands

This module covers `workhub cli confluence ...` commands.

## Example Input Files

Markdown page file `/tmp/workhub-page.md`:

```markdown
# Roadmap

Status: draft

- Milestone A
- Milestone B
```

Storage-format page file `/tmp/workhub-page.storage`:

```html
<p>Status: draft</p><ul><li>Milestone A</li><li>Milestone B</li></ul>
```

Comment file `/tmp/workhub-comment.md`:

```markdown
Please review the updated roadmap section.
```

Attachment path examples:

```text
/tmp/workhub-report.pdf
/tmp/workhub-screenshot.png
```

## Content, Labels, And Users

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence content search --query <text> [--limit <n>] [--spaces <csv>]` | Search Confluence content. | `workhub cli confluence content search --query 'roadmap' --spaces ENG,DOCS --limit 10` | Search result table by default; result list and paging fields with `--json`. |
| `cli confluence content label list <content-id>` | List labels for content. | `workhub cli confluence content label list 123456` | Label table by default; label list with `--json`. |
| `cli confluence content label add <content-id> <label>` | Add a label to content. | `workhub cli confluence content label add 123456 roadmap` | Operation summary by default; added label or operation result with `--json`. |
| `cli confluence user search --query <text> [--limit <n>] [--group <name>]` | Search Confluence users, optionally within a group. | `workhub cli confluence user search --query Ada --limit 10 --group confluence-users` | User table by default; user list with `--json`. |

## Page Reads

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence page get (--id <page-id> \| --space <key> --title <title>) [--include-metadata <bool>] [--markdown <bool>]` | Get a page by ID or by space and title. | `workhub cli confluence page get --space ENG --title 'Roadmap' --markdown true` | Page fields by default; page object with `--json`. With `--markdown true`, content is converted to Markdown. |
| `cli confluence page children <parent-id> [--expand <csv>] [--limit <n>] [--include-content] [--markdown <bool>] [--start <n>] [--include-folders <bool>]` | List child pages and optionally include content. | `workhub cli confluence page children 123456 --include-content --markdown true --limit 20` | Child page table by default; child page list with `--json`. |
| `cli confluence page tree <space-key> [--limit <n>]` | Get a space page tree. | `workhub cli confluence page tree ENG --limit 100` | Page tree or page list by default; tree data with `--json`. |

## Page Writes And Moves

Content input uses one source at a time: `--content`, `--content-file`, or `--content-stdin`. `--format` accepts `markdown`, `wiki`, or `storage`.

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence page create --space <key> --title <title> (--content <text> \| --content-file <path> \| --content-stdin) [--parent-id <id>] [--format <markdown\|wiki\|storage>] [--include-content] [--emoji <emoji>]` | Create a page. | `workhub cli confluence page create --space ENG --title 'Roadmap' --content-file /tmp/workhub-page.md --format markdown --parent-id 123456 --include-content` | Created page summary by default, usually including id, title, and URL. With `--include-content`, content fields are included. `--json` returns the page object. |
| `cli confluence page update <page-id> --title <title> (--content <text> \| --content-file <path> \| --content-stdin) [--minor-edit <bool>] [--version-comment <text>] [--parent-id <id>] [--format <markdown\|wiki\|storage>] [--include-content] [--emoji <emoji>]` | Update a page title and content. | `workhub cli confluence page update 123456 --title 'Roadmap' --content-file /tmp/workhub-page.md --format markdown --minor-edit true --version-comment 'Refresh roadmap'` | Updated page summary by default. With `--include-content`, content fields are included. `--json` returns the page object. |
| `cli confluence page delete <page-id>` | Delete a page. | `workhub cli confluence page delete 123456` | Delete operation summary by default; operation result with `--json`. |
| `cli confluence page move <page-id> [--target-parent-id <id>] [--target-space <key>] [--position <append\|before\|after>]` | Move a page to a target parent or space. | `workhub cli confluence page move 123456 --target-parent-id 789012 --position append` | Move operation summary by default; operation result or moved page data with `--json`. |

## Page Comments

Comment body input uses one source at a time: `--body`, `--body-file`, or `--body-stdin`.

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence page comment list <page-id>` | List page comments. | `workhub cli confluence page comment list 123456` | Comment table by default; comment list with `--json`. |
| `cli confluence page comment add <page-id> (--body <text> \| --body-file <path> \| --body-stdin)` | Add a page comment. | `workhub cli confluence page comment add 123456 --body-file /tmp/workhub-comment.md` | New comment summary by default; comment object with `--json`. |
| `cli confluence page comment reply <page-id> <comment-id> (--body <text> \| --body-file <path> \| --body-stdin)` | Reply to an existing comment. | `workhub cli confluence page comment reply 123456 98765 --body 'Thanks, updated.'` | Reply summary by default; comment object with `--json`. |

## Page Versions And Analytics

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence page version get <page-id> <version> [--markdown <bool>]` | Get a specific page version. | `workhub cli confluence page version get 123456 3 --markdown true` | Version fields by default; version object with `--json`. With `--markdown true`, content is converted to Markdown. |
| `cli confluence page version diff <page-id> --from <version> --to <version> [--context-lines <n>]` | Compare two page versions. | `workhub cli confluence page version diff 123456 --from 2 --to 3 --context-lines 5` | Diff text or summary by default; diff object with `--json`. |
| `cli confluence page analytics views <page-id> [--from <date>] [--to <date>] [--include-title <bool>]` | Get page view analytics. | `workhub cli confluence page analytics views 123456 --from 2026-06-01 --to 2026-06-12 --include-title true` | View metrics by default; analytics object with `--json`. |

## Attachments

Upload paths are resolved on the machine or container running the CLI. Download commands return attachment content in command output; they do not automatically write files.

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli confluence attachment upload <content-id> <file-path> [--comment <text>] [--minor-edit <bool>]` | Upload one attachment. | `workhub cli confluence attachment upload 123456 /tmp/workhub-report.pdf --comment 'Report' --minor-edit true` | Uploaded attachment summary by default; attachment object with `--json`. |
| `cli confluence attachment upload-batch <content-id> --files <csv> [--comment <text>] [--minor-edit <bool>]` | Upload multiple attachments. | `workhub cli confluence attachment upload-batch 123456 --files /tmp/a.png,/tmp/b.png --comment 'Screenshots' --minor-edit true` | Per-file upload results by default; batch upload result with `--json`. |
| `cli confluence attachment list <content-id> [--filename-contains <text>] [--media-type <type>] [--start <n>] [--limit <n>]` | List content attachments with optional filters. | `workhub cli confluence attachment list 123456 --filename-contains report --media-type application/pdf --limit 20` | Attachment table by default; attachment list and paging fields with `--json`. |
| `cli confluence attachment download <attachment-id> [--max-bytes <n>]` | Download one attachment into command output. | `workhub cli confluence attachment download att123 --max-bytes 1048576` | Attachment metadata and bounded content fields by default; download object with `--json`. |
| `cli confluence attachment download-content <content-id> [--filename-contains <text>] [--media-type <type>] [--max-bytes <n>] [--limit <n>]` | Download matching attachments for content into command output. | `workhub cli confluence attachment download-content 123456 --media-type image/png --max-bytes 1048576 --limit 5` | Matching attachments and content fields by default; batch download object with `--json`. |
| `cli confluence attachment delete <attachment-id>` | Delete an attachment. | `workhub cli confluence attachment delete att123` | Delete operation summary by default; operation result with `--json`. |
| `cli confluence attachment images <content-id> [--max-bytes <n>]` | Get image attachments for content. | `workhub cli confluence attachment images 123456 --max-bytes 1048576` | Image attachment table and available content fields by default; image attachment object with `--json`. |
