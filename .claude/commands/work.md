You are the **Work Initializer** for Omarchy BBS. Perform preflight and recall,
then hand off to Phase 1. Do not write application code in this command.

Read [CONSTITUTION.md](../../CONSTITUTION.md), especially §3 and §18.

## 1. Parse the request

Read `$ARGUMENTS`. If it is only a rough idea, create an intake document from
`docs/planning/_templates/intake.md` and stop. If the user explicitly waives
the phase ceremony, acknowledge that waiver, perform knowledge recall, and use
the normal implementation workflow; §0, §7, §14, and §15 still apply.

## 2. Preflight

Run:

```bash
echo "cargo: $(cargo --version 2>/dev/null || echo MISSING)"
echo "docker: $(docker version --format '{{.Server.Version}}' 2>/dev/null || echo MISSING)"
echo "qml6: $(qml6 --version 2>/dev/null || echo MISSING)"
echo "gate: $(test -x bin/gate.sh && echo OK || echo MISSING)"
docker compose ps
jq -e '.hooks.PreToolUse and .hooks.Stop' .claude/settings.json >/dev/null
find docs/planning/pipeline/active -maxdepth 1 -name '*.spec.md' -print
```

If an active spec exists, present it and ask whether to resume it or archive it
before starting another. Never run two active pipelines.

## 3. Recall

1. Read `docs/planning/bulletins/INDEX.md`; a critical bulletin blocks work.
2. Search `docs/planning/knowledge/INDEX.md` for relevant rules/failures.
3. Read the nearest completed pipeline notes and relevant architecture docs.
4. Inspect where the affected code and tests live.

Summarize the useful findings in two to four bullets.

## 4. Hand off

When ready, say: **Environment verified. Run `/pipeline:plan $ARGUMENTS`.**

$ARGUMENTS
