import type { EngineInterface, Register } from 'claude-code'

const TOOL = 'mcp__self-compact__compact'

type Job = { instructions?: string; resume?: string; tries: number; rounds: number }
let pending: Job | undefined

const text = (v: unknown) => (typeof v === 'string' && v.trim() ? v.trim() : undefined)

const DESCRIPTION = `Compact (summarize) THIS conversation yourself, exactly as if the user typed /compact.

Compaction cannot run in the middle of a turn, so this call only queues it: it runs right after your current reply ends. After calling it, finish the turn promptly with a 1-2 line note; everything up to that point goes into the summary.

Call it when AgentRig requires compaction after a merge, or when the user asks for it.

Arguments:
- instructions: what the summary must keep (decisions made, file paths, open TODOs, the next step, user preferences). Written like the text after /compact.
- resume: if work should continue after compaction, the prompt to send yourself once the summary is in place (e.g. "Continue with step 3: wire the API client into the settings page"). Leave it out to stop and wait for the user.

Main conversation only; subagents cannot use it.`

async function compactNow($: EngineInterface, job: Job): Promise<void> {
  try {
    const done = await $.session.compact(job.instructions ? { instructions: job.instructions } : {})
    $.ui.status(undefined)
    if (done.skip !== undefined) {
      $.ui.toast(`self-compact: skipped (${done.skip})`)
      return
    }
    const sizes =
      done.tokensBefore !== undefined && done.tokensAfter !== undefined
        ? ` ${Math.round(done.tokensBefore / 1000)}k → ${Math.round(done.tokensAfter / 1000)}k tokens`
        : ''
    $.ui.toast(`self-compact: conversation compacted${sizes}`)
    if (job.resume) {
      await $.prompt.submit({
        text: `[self-compact] The conversation was just compacted at your own request. Continue: ${job.resume}`,
      })
    }
  } catch {
    // Rejects while a turn runs (the turn has not fully wound down, or a queued prompt started one).
    job.tries += 1
    if (job.tries <= 5) {
      $.clock.after(1000, () => void compactNow($, job))
    } else if (job.rounds < 3 && pending === undefined) {
      job.tries = 0
      pending = job // try again when the next turn ends
      $.ui.status('self-compact: waiting for the turn to end')
    } else {
      $.ui.status(undefined)
      $.ui.toast('self-compact: could not compact, a turn kept running')
    }
  }
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    await $.tool.register({
      name: 'compact',
      description: DESCRIPTION,
      isDeferred: false,
      inputSchema: {
        type: 'object',
        properties: {
          instructions: {
            type: 'string',
            description: 'What the summary must keep or stress, as typed after /compact.',
          },
          resume: {
            type: 'string',
            description: 'Prompt to send yourself after compaction to keep working. Omit to wait for the user.',
          },
        },
      },
    })
    return next(e)
  })

  on('tool.run', { tool: TOOL }, async ($, e) => {
    if (e.agentId !== undefined) {
      return { deny: 'Only the main conversation can compact itself; a subagent cannot.' }
    }
    const args = e as unknown as Record<string, unknown>
    pending = { instructions: text(args.instructions), resume: text(args.resume), tries: 0, rounds: 0 }
    $.ui.status('self-compact: compaction queued for the end of this turn')
    return {
      result:
        'Compaction queued: it runs as soon as this turn ends. Wrap up now with a 1-2 line note to the user' +
        (pending.resume ? '; after the summary you will be prompted to continue.' : '.'),
    }
  }).catch(() => ({ deny: 'self-compact: could not queue the compaction.' }))

  on('turn.complete', async ($, e, next) => {
    const answered = await next(e)
    if (e.agentId !== undefined || pending === undefined) return answered
    const job = pending
    pending = undefined
    if (e.isAborted) {
      $.ui.status(undefined)
      $.ui.toast('self-compact: turn interrupted, compaction cancelled')
      return answered
    }
    job.rounds += 1
    $.ui.status('self-compact: compacting…')
    $.clock.after(300, () => void compactNow($, job))
    return answered
  })
}
