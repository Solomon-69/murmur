import Dispatch
import Foundation
import FoundationModels

private let PLAIN_OUTPUT_MARKER = "[[plain]]"

@available(macOS 26.0, *)
@Generable
private struct CleanedTranscript: Sendable {
    let cleanedText: String
}

// MARK: - Swift implementation for Apple LLM integration
// This file is compiled via Cargo build script for Apple Silicon targets

private typealias ResponsePointer = UnsafeMutablePointer<AppleLLMResponse>

private func duplicateCString(_ text: String) -> UnsafeMutablePointer<CChar>? {
    return text.withCString { basePointer in
        guard let duplicated = strdup(basePointer) else {
            return nil
        }
        return duplicated
    }
}

private func truncatedText(_ text: String, limit: Int) -> String {
    guard limit > 0 else { return text }
    let words = text.split(
        maxSplits: .max,
        omittingEmptySubsequences: true,
        whereSeparator: { $0.isWhitespace || $0.isNewline }
    )
    if words.count <= limit {
        return text
    }
    return words.prefix(limit).joined(separator: " ")
}

@_cdecl("is_apple_intelligence_available")
public func isAppleIntelligenceAvailable() -> Int32 {
    guard #available(macOS 26.0, *) else {
        return 0
    }

    let model = SystemLanguageModel.default
    switch model.availability {
    case .available:
        return 1
    case .unavailable:
        return 0
    }
}

/// Sessions made ahead of time, one per instruction text. `prewarm()` loads
/// the on-device model and reads the instructions while the person is still
/// talking, so the rewrite starts generating at once. Each session is used
/// for one answer only (a session keeps its history), then dropped.
@available(macOS 26.0, *)
private final class SessionCache: @unchecked Sendable {
    static let shared = SessionCache()
    private let lock = NSLock()
    private var sessions: [String: LanguageModelSession] = [:]

    func take(_ instructions: String) -> LanguageModelSession? {
        lock.lock()
        defer { lock.unlock() }
        return sessions.removeValue(forKey: instructions)
    }

    func peek(_ instructions: String) -> LanguageModelSession? {
        lock.lock()
        defer { lock.unlock() }
        return sessions[instructions]
    }

    func put(_ instructions: String, _ session: LanguageModelSession) {
        lock.lock()
        defer { lock.unlock() }
        // Only a handful of instruction texts exist; never let this grow.
        if sessions.count >= 4 {
            sessions.removeAll()
        }
        sessions[instructions] = session
    }
}

private func stripPlainMarker(_ raw: String) -> (String, Bool) {
    if raw.hasPrefix(PLAIN_OUTPUT_MARKER) {
        return (String(raw.dropFirst(PLAIN_OUTPUT_MARKER.count)), true)
    }
    return (raw, false)
}

/// Loads the model and prepares a session for `systemPrompt` in the
/// background. Returns 1 when a warm-up was started.
@_cdecl("prewarm_apple_intelligence")
public func prewarmAppleIntelligence(_ systemPrompt: UnsafePointer<CChar>) -> Int32 {
    guard #available(macOS 26.0, *) else {
        return 0
    }
    let model = SystemLanguageModel.default
    guard model.availability == .available else {
        return 0
    }
    let (instructions, _) = stripPlainMarker(String(cString: systemPrompt))
    if let existing = SessionCache.shared.peek(instructions) {
        existing.prewarm()
        return 1
    }
    let session = LanguageModelSession(model: model, instructions: instructions)
    session.prewarm()
    SessionCache.shared.put(instructions, session)
    return 1
}

/// Room for the answer: generous for edits that grow the text, yet a hard
/// stop if the model starts repeating itself.
private func responseTokenCap(for userContent: String) -> Int {
    return max(800, userContent.count * 3 / 8 + 300)
}

@_cdecl("process_text_with_system_prompt_apple")
public func processTextWithSystemPrompt(
    _ systemPrompt: UnsafePointer<CChar>,
    _ userContent: UnsafePointer<CChar>,
    maxTokens: Int32
) -> UnsafeMutablePointer<AppleLLMResponse> {
    // A prompt starting with PLAIN_OUTPUT_MARKER wants free-form text
    // (Turn into a prompt, Command Mode); the one-field structured answer is
    // slower and would flatten Markdown.
    let (swiftSystemPrompt, plainOutput) = stripPlainMarker(String(cString: systemPrompt))
    let swiftUserContent = String(cString: userContent)
    let responsePtr = ResponsePointer.allocate(capacity: 1)
    responsePtr.initialize(to: AppleLLMResponse(response: nil, success: 0, error_message: nil))

    guard #available(macOS 26.0, *) else {
        responsePtr.pointee.error_message = duplicateCString(
            "Apple Intelligence requires macOS 26 or newer."
        )
        return responsePtr
    }

    let model = SystemLanguageModel.default
    guard model.availability == .available else {
        responsePtr.pointee.error_message = duplicateCString(
            "Apple Intelligence is not currently available on this device."
        )
        return responsePtr
    }

    let tokenLimit = max(0, Int(maxTokens))
    let semaphore = DispatchSemaphore(value: 0)

    // Thread-safe container to pass results from async task back to calling thread
    final class ResultBox: @unchecked Sendable {
        var response: String?
        var error: String?
    }
    let box = ResultBox()

    Task.detached(priority: .userInitiated) {
        defer { semaphore.signal() }
        do {
            // A warmed-up session for these instructions when one is ready.
            var session = SessionCache.shared.take(swiftSystemPrompt)
                ?? LanguageModelSession(model: model, instructions: swiftSystemPrompt)
            if session.isResponding {
                session = LanguageModelSession(model: model, instructions: swiftSystemPrompt)
            }
            // Low temperature: the rewrite should be faithful, not creative.
            let options = GenerationOptions(
                temperature: 0.2,
                maximumResponseTokens: responseTokenCap(for: swiftUserContent)
            )
            var output: String

            if plainOutput {
                output = try await session.respond(to: swiftUserContent, options: options).content
            } else {
                do {
                    let structured = try await session.respond(
                        to: swiftUserContent,
                        generating: CleanedTranscript.self,
                        options: options
                    )
                    output = structured.content.cleanedText
                } catch {
                    let retry = LanguageModelSession(model: model, instructions: swiftSystemPrompt)
                    let fallbackGeneration = try await retry.respond(
                        to: swiftUserContent,
                        options: options
                    )
                    output = fallbackGeneration.content
                }
            }

            if tokenLimit > 0 {
                output = truncatedText(output, limit: tokenLimit)
            }
            box.response = output
        } catch {
            box.error = error.localizedDescription
        }
    }

    semaphore.wait()

    // Write to responsePtr on the calling thread after task completes
    if let response = box.response {
        responsePtr.pointee.response = duplicateCString(response)
        responsePtr.pointee.success = 1
    } else {
        responsePtr.pointee.error_message = duplicateCString(box.error ?? "Unknown error")
    }

    return responsePtr
}

@_cdecl("free_apple_llm_response")
public func freeAppleLLMResponse(_ response: UnsafeMutablePointer<AppleLLMResponse>?) {
    guard let response = response else { return }

    if let responseStr = response.pointee.response {
        free(UnsafeMutablePointer(mutating: responseStr))
    }

    if let errorStr = response.pointee.error_message {
        free(UnsafeMutablePointer(mutating: errorStr))
    }

    response.deallocate()
}