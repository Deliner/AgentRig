from __future__ import annotations

import re
import tokenize
from pathlib import Path

# DECISION: D002

DECISION_ROW = re.compile(r"^\| \[(D\d{3})\]\((Decisions/\d{3}\.md)\) \| (.+) \| (.+) \|$")
INVARIANT_ROW = re.compile(
    r"^\| \[(I\d{3})\]\((Invariants/\d{3}\.md)\) \| (.+) "
    r"\| \[([A-Za-z_]\w*)\]\(([^)]+\.py)\) \|$"
)
LINK = re.compile(r"\[[^]]+\]\(([^)#]+)(?:#[^)]+)?\)")


def table_rows(path: Path, pattern: re.Pattern[str]) -> list[re.Match[str]]:
    return [
        match
        for line in path.read_text(encoding="utf-8").splitlines()
        if (match := pattern.match(line))
    ]


def shell_comments(source: str) -> list[tuple[int, str]]:
    comments: list[tuple[int, str]] = []
    quote: str | None = None
    escaped = False
    substitutions: list[tuple[str | None, int]] = []
    line = 1
    index = 0
    while index < len(source):
        character = source[index]
        if escaped:
            escaped = False
        elif character == "\\" and quote != "'":
            escaped = True
        elif quote and character == quote:
            quote = None
        elif quote != "'" and character == "`":
            if substitutions and substitutions[-1][1] == 0:
                quote, _ = substitutions.pop()
            else:
                substitutions.append((quote, 0))
                quote = None
        elif quote != "'" and source[index : index + 2] == "$(":
            substitutions.append((quote, 1))
            quote = None
            index += 1
        elif not quote and character in {"'", '"'}:
            quote = character
        elif not quote and substitutions and substitutions[-1][1] > 0 and character == "(":
            previous_quote, depth = substitutions[-1]
            substitutions[-1] = (previous_quote, depth + 1)
        elif not quote and substitutions and substitutions[-1][1] > 0 and character == ")":
            previous_quote, depth = substitutions[-1]
            if depth == 1:
                substitutions.pop()
                quote = previous_quote
            else:
                substitutions[-1] = (previous_quote, depth - 1)
        elif (
            not quote
            and character == "#"
            and (index == 0 or source[index - 1].isspace() or source[index - 1] in ";|&()<>")
        ):
            end = source.find("\n", index)
            end = len(source) if end == -1 else end
            comments.append((line, source[index:end]))
            index = end - 1
        if character == "\n":
            line += 1
        index += 1
    return comments


def source_comments(path: Path) -> list[tuple[int, str]] | None:
    try:
        first_line = path.read_bytes().split(b"\n", 1)[0].decode("utf-8")
    except UnicodeDecodeError:
        return None
    if path.suffix == ".sh" or (
        first_line.startswith("#!") and ("sh" in first_line or "bash" in first_line)
    ):
        return shell_comments(path.read_text(encoding="utf-8"))
    if path.suffix != ".py":
        return None
    tokens = tokenize.generate_tokens(path.open(encoding="utf-8").readline)
    return [(token.start[0], token.string) for token in tokens if token.type == tokenize.COMMENT]
