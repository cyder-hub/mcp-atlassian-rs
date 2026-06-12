# GitLab Commands

This module covers `workhub cli gitlab ...` commands.

The `<project>` argument can be a numeric project ID or a full path such as `group/project` or `group/subgroup/project`.

## Example Input Files

MR description file `/tmp/workhub-mr-description.md`:

```markdown
## Summary

Updates the user-facing workflow.

## Notes

- Checked the relevant command output
- Confirmed the target branch and reviewer list
```

MR note file `/tmp/workhub-mr-note.md`:

```markdown
Please document the empty assignee case.
```

Merge commit message files are not supported directly. Use `--merge-commit-message <text>` or `--squash-commit-message <text>`.

## Users And Projects

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli gitlab user current` | Show the GitLab user for the configured token. | `workhub cli gitlab user current` | Text user fields by default; user object with `--json`. |
| `cli gitlab project get <project>` | Get project details. | `workhub cli gitlab project get group/project` | Text project fields by default; project object with `--json`. |

## Merge Request Reads

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli gitlab mr list <project> [--state <state>] [--author <username>] [--reviewer <username>] [--source-branch <branch>] [--target-branch <branch>] [--labels <csv>] [--page <n>] [--per-page <n>]` | List project merge requests with optional filters. | `workhub cli gitlab mr list group/project --state opened --labels backend,urgent --per-page 20` | MR table by default; MR list with `--json`. |
| `cli gitlab mr get <project> <iid> [--include-diverged-commits-count <bool>] [--include-rebase-in-progress <bool>]` | Get one merge request. | `workhub cli gitlab mr get group/project 7 --include-diverged-commits-count true --include-rebase-in-progress true` | MR fields by default; MR object with `--json`. |
| `cli gitlab mr commits <project> <iid> [--page <n>] [--per-page <n>]` | List merge request commits. | `workhub cli gitlab mr commits group/project 7 --per-page 20` | Commit table by default; commit list with `--json`. |
| `cli gitlab mr diffs <project> <iid> [--max-diff-bytes <n>] [--page <n>] [--per-page <n>]` | List merge request diffs with optional byte limit. | `workhub cli gitlab mr diffs group/project 7 --max-diff-bytes 65536 --per-page 20` | Diff summary or file rows by default; diff list with `--json`, bounded by `--max-diff-bytes`. |
| `cli gitlab mr pipelines <project> <iid> [--page <n>] [--per-page <n>]` | List merge request pipelines. | `workhub cli gitlab mr pipelines group/project 7 --per-page 20` | Pipeline table by default; pipeline list with `--json`. |
| `cli gitlab mr approval get <project> <iid>` | Show merge request approval state. | `workhub cli gitlab mr approval get group/project 7` | Approval rules and approver summary by default; approval state object with `--json`. |

## Merge Request Writes

Description and body inputs use one source at a time: inline, file, or stdin.

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli gitlab mr create <project> --source <branch> --target <branch> --title <title> [--description <text> \| --description-file <path> \| --description-stdin] [--remove-source-branch <bool>] [--squash <bool>] [--assignee-ids <csv>] [--reviewer-ids <csv>] [--labels <csv>]` | Create a merge request. | `workhub cli gitlab mr create group/project --source feature/workhub-cli --target main --title 'Update user workflow' --description-file /tmp/workhub-mr-description.md --reviewer-ids 101,102 --labels cli,docs --remove-source-branch true` | New MR summary by default, usually including iid, title, state, and web URL; MR object with `--json`. |
| `cli gitlab mr update <project> <iid> [--title <title>] [--description <text> \| --description-file <path> \| --description-stdin] [--state-event <event>] [--labels <csv>] [--add-labels <csv>] [--remove-labels <csv>] [--reviewer-ids <csv>] [--assignee-ids <csv>] [--target-branch <branch>]` | Update merge request title, description, labels, reviewers, assignees, target branch, or state event. | `workhub cli gitlab mr update group/project 7 --title 'Update user workflow docs' --add-labels ready --remove-labels draft` | Updated MR summary by default; MR object with `--json`. |
| `cli gitlab mr note add <project> <iid> (--body <text> \| --body-file <path> \| --body-stdin)` | Add a merge request note. | `workhub cli gitlab mr note add group/project 7 --body-file /tmp/workhub-mr-note.md` | New note summary by default; note object with `--json`. |
| `cli gitlab mr discussion reply <project> <iid> <discussion-id> (--body <text> \| --body-file <path> \| --body-stdin)` | Reply to a merge request discussion. | `workhub cli gitlab mr discussion reply group/project 7 abcdef123 --body 'Addressed in the latest push.'` | Reply summary by default; discussion or note object with `--json`. |
| `cli gitlab mr discussion resolve <project> <iid> <discussion-id> --resolved <bool>` | Mark a discussion resolved or unresolved. | `workhub cli gitlab mr discussion resolve group/project 7 abcdef123 --resolved true` | Discussion state summary by default; operation result with `--json`. |
| `cli gitlab mr approval set <project> <iid> --action <approve\|unapprove>` | Approve or unapprove a merge request. | `workhub cli gitlab mr approval set group/project 7 --action approve` | Approval operation summary by default; approval result with `--json`. |
| `cli gitlab mr merge <project> <iid> --sha <sha> [--auto-merge <bool>] [--squash <bool>] [--remove-source-branch <bool>] [--merge-commit-message <text>] [--squash-commit-message <text>]` | Merge a merge request. Requires a reviewed head SHA. | `workhub cli gitlab mr merge group/project 7 --sha abc123def456 --auto-merge false --squash true --remove-source-branch true --merge-commit-message 'Merge MR !7'` | Merged MR summary by default; MR or merge result object with `--json`. |

## Safety Notes

- Run `cli gitlab mr get <project> <iid>` before merging to verify state, target branch, and latest SHA.
- `cli gitlab mr approval set` accepts only `approve` or `unapprove`.
- Project-scoped commands respect `GITLAB_PROJECTS_FILTER` before sending upstream requests.
