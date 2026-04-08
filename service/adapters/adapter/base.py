"""
Base adapter class for LLM providers.
"""
from abc import ABC, abstractmethod
from typing import List, Dict, Any, AsyncIterator, Optional
from pydantic import BaseModel


class Message(BaseModel):
    role: str
    content: str


class ChatRequest(BaseModel):
    provider: str
    model: str
    messages: List[Message]
    temperature: float = 0.7
    max_tokens: Optional[int] = None
    stream: bool = False
    extra: Dict[str, str] = Field(default_factory=dict)


class ChatResponse(BaseModel):
    id: str
    model: str
    message: Message
    usage: Dict[str, int]
    latency_ms: int


class ChatChunk(BaseModel):
    delta: str
    finished: bool
    usage: Optional[Dict[str, int]] = None
    finish_reason: Optional[str] = None


class EmbeddingsRequest(BaseModel):
    model: str
    inputs: List[str]


class EmbeddingsResponse(BaseModel):
    embeddings: List[List[float]]


class BaseAdapter(ABC):
    """Abstract base class for all LLM provider adapters."""

    @property
    @abstractmethod
    def provider_name(self) -> str:
        """Return the provider name (e.g., 'openai', 'anthropic')."""
        pass

    @abstractmethod
    async def chat(self, request: ChatRequest) -> ChatResponse:
        """Send a chat request to the provider."""
        pass

    @abstractmethod
    async def chat_stream(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        """Send a streaming chat request to the provider."""
        pass

    @abstractmethod
    async def embeddings(self, request: EmbeddingsRequest) -> EmbeddingsResponse:
        """Generate embeddings."""
        pass

    async def health_check(self) -> bool:
        """Check if the provider is healthy."""
        return True
