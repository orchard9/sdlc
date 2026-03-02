import type {
  RawRunEvent,
  PairedEvent,
  PairedToolExchange,
  PairedAssistantText,
} from '@/lib/types'

/**
 * Convert the flat event array from the telemetry API into structured
 * PairedEvent objects for rendering in the activity feed.
 *
 * Pairing rules:
 * - 'init' → PairedInitEvent
 * - 'assistant' text → PairedAssistantText (text + tools merged in order)
 * - 'tool_progress' → accumulated into the current pending tool exchange
 * - 'tool_summary' → summary for the current pending tool exchange
 * - 'result' → PairedRunResult
 * - Other event types → skipped
 *
 * Because the existing event format from message_to_event does not emit
 * separate tool_call/tool_result pairs with IDs (it uses a simpler format),
 * we group tool activity from each assistant message: the tool names come
 * from 'assistant' events, progress from 'tool_progress', and summary from
 * 'tool_summary'. Each tool use in an assistant message becomes one
 * PairedToolExchange.
 */
export function pairEvents(
  events: RawRunEvent[],
  prompt?: string | null
): PairedEvent[] {
  const result: PairedEvent[] = []

  // We process events sequentially. The current event format is:
  //   init → assistant (text + tools[]) → tool_progress* → tool_summary? → user → assistant → ...
  //
  // We build one ToolExchange per tool entry in the assistant's tools[] array,
  // collecting progress and summary as they arrive before the next assistant event.

  let pendingTools: PairedToolExchange[] = []
  let lastAssistantText: string | null = null

  const flushPendingTools = () => {
    for (const t of pendingTools) {
      result.push(t)
    }
    pendingTools = []
  }

  const flushAssistantText = () => {
    if (lastAssistantText != null && lastAssistantText.trim().length > 0) {
      const block: PairedAssistantText = {
        kind: 'assistant_text',
        text: lastAssistantText,
      }
      result.push(block)
    }
    lastAssistantText = null
  }

  for (const event of events) {
    switch (event.type) {
      case 'init': {
        result.push({
          kind: 'init',
          event,
          prompt,
        })
        break
      }

      case 'assistant': {
        // Flush any pending tools from the previous round before starting fresh
        flushPendingTools()
        flushAssistantText()

        // Text content
        if (event.text && event.text.trim().length > 0) {
          lastAssistantText = event.text
        }

        // Tool calls — each becomes a pending exchange
        if (event.tools && event.tools.length > 0) {
          for (const tool of event.tools) {
            pendingTools.push({
              kind: 'tool_exchange',
              toolName: tool.name,
              input: tool.input,
              isError: false,
            })
          }
        }

        // If there are no tools, flush the text now
        if (!event.tools || event.tools.length === 0) {
          flushAssistantText()
        }
        break
      }

      case 'tool_progress': {
        // Update the last pending tool with elapsed time
        if (pendingTools.length > 0) {
          const last = pendingTools[pendingTools.length - 1]
          last.elapsed_seconds = event.elapsed_seconds
        }
        break
      }

      case 'tool_summary': {
        // Update the last pending tool with the summary
        if (pendingTools.length > 0) {
          const last = pendingTools[pendingTools.length - 1]
          last.summary = event.summary
        }
        break
      }

      case 'user': {
        // User message means the tool results came back; flush pending tools
        flushAssistantText()
        flushPendingTools()
        break
      }

      case 'result': {
        flushAssistantText()
        flushPendingTools()
        result.push({
          kind: 'run_result',
          isError: event.is_error ?? false,
          cost_usd: event.cost_usd,
          turns: event.turns,
          text: typeof event.text === 'string' ? event.text : undefined,
        })
        break
      }

      case 'error': {
        flushAssistantText()
        flushPendingTools()
        result.push({
          kind: 'run_result',
          isError: true,
          text: event.message,
        })
        break
      }

      default:
        // skip: system, stream_event, auth_status, status
        break
    }
  }

  // Flush anything remaining
  flushAssistantText()
  flushPendingTools()

  return result
}
