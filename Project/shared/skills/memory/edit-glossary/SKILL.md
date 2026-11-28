---
name: edit-glossary
description: Add, change or remove a project word in the glossary only after the user explicitly agrees to it in chat, in the entry form and within the limits the rig checks.
---

# Keep each project word in the meaning the user gives it

The glossary holds the project's words with the meaning the user gives them, so
that every session reads them as the user does. Each session receives it in full
at its start.

Add, change or remove a word only after the user explicitly agrees to that entry
in chat. Show the entry first; a word you noticed, a meaning that seems likelier
or the repair of a refused commit remains a proposal until the user agrees. The
rig does not check the agreement.

Give each word one entry that states its meaning in this project, not its history
or examples. When the meaning changes, replace the entry instead of adding one.

## Repository binding

The glossary is Glossary.md in the memory directory. Its first line is a `# `
header; each later nonblank line is `- **word** — meaning` with a word and a
meaning. A word appears once, and letter case does not tell words apart. The
memory check refuses another line or a repeated word, naming the file and line;
lint keeps the file within 100 nonblank lines, counting the header, of at most
120 characters.
