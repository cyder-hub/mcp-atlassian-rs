---
name: workhub-rs
description: End-user manual for the installed workhub CLI. Use when explaining how to configure credentials, run `workhub cli ...`, choose Jira/Confluence/GitLab resource commands, prepare file/stdin/JSON inputs, and interpret text, JSON, and error output.
---

# Workhub RS CLI Manual

This skill is for users who already have the `workhub` binary and want to use its resource-oriented CLI. It covers `workhub cli ...` only.

This skill only covers CLI command usage, CLI inputs, and CLI return behavior.

## Reference Map

Read only the module needed for the task:

1. CLI configuration, global flags, input forms, output, and errors: [references/runtime.md](references/runtime.md).
2. Jira issues, projects, fields, Agile, service desk, attachments, and related data: [references/jira.md](references/jira.md).
3. Confluence search, pages, comments, labels, users, versions, analytics, and attachments: [references/confluence.md](references/confluence.md).
4. GitLab users, projects, and merge requests: [references/gitlab.md](references/gitlab.md).

## CLI Shape

Use this top-level shape:

```bash
workhub cli [--env-file <path>] [--json] [--pretty] <provider> <resource> <action> ...
```

The provider entry points are:

```bash
workhub cli jira ...
workhub cli confluence ...
workhub cli gitlab ...
```

If the binary is not on `PATH`, replace `workhub` with its absolute path.

Use built-in help for exact flag spelling:

```bash
workhub cli --help
workhub cli jira issue search --help
workhub cli confluence page create --help
workhub cli gitlab mr merge --help
```

## Safety

Use read commands first when identifying IDs, current state, target branches, page locations, or destructive-operation targets. Only run write, delete, move, approval, or merge commands when that action is explicitly intended.
