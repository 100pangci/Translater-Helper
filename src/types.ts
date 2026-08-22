export type ReasoningEffort = "default" | "low" | "medium" | "high";

export interface AppConfig {
  provider: "openai" | "anthropic";
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  reasoningEffort: ReasoningEffort | null;
  systemPrompt: string;
}

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface SessionTurn {
  question: string;
  answer: string;
  isFollowUp: boolean;
}

export interface StreamPayload {
  requestId: string;
  delta?: string;
  message?: string;
}