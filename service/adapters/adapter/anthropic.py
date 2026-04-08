"""
Anthropic API adapter.
"""
import os
import time
from typing import AsyncIterator

import httpx

from .base import (
    BaseAdapter,
    ChatRequest,
    ChatResponse,
    ChatChunk,
    EmbeddingsRequest,
    EmbeddingsResponse,
    Message,
)


class AnthropicAdapter(BaseAdapter):
    """Adapter for Anthropic Claude API."""

    BASE_URL = "https://api.anthropic.com/v1"

    def __init__(self):
        self.api_key = os.getenv("ANTHROPIC_API_KEY", "")
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY environment variable is not set")
        self.client = httpx.AsyncClient(timeout=60.0)

    @property
    def provider_name(self) -> str:
        return "anthropic"

    def _convert_messages(self, messages):
        """Convert messages format for Anthropic."""
        anthropic_messages = []
        system_prompt = ""

        for msg in messages:
            if msg.role == "system":
                system_prompt = msg.content
            elif msg.role == "user":
                anthropic_messages.append({
                    "role": "user",
                    "content": msg.content,
                })
            elif msg.role == "assistant":
                anthropic_messages.append({
                    "role": "assistant",
                    "content": msg.content,
                })

        return anthropic_messages, system_prompt

    async def chat(self, request: ChatRequest) -> ChatResponse:
        """Send a chat request to Anthropic."""
        url = f"{self.BASE_URL}/messages"

        anthropic_messages, system_prompt = self._convert_messages(request.messages)

        headers = {
            "x-api-key": self.api_key,
            "anthropic-version": "2023-06-01",
            "Content-Type": "application/json",
        }

        payload = {
            "model": request.model,
            "messages": anthropic_messages,
            "max_tokens": request.max_tokens or 4096,
            "temperature": request.temperature,
        }

        if system_prompt:
            payload["system"] = system_prompt

        start = time.time()
        response = await self.client.post(url, json=payload, headers=headers)
        latency_ms = int((time.time() - start) * 1000)

        response.raise_for_status()
        data = response.json()

        return ChatResponse(
            id=f"msg_{data['id']}",
            model=data["model"],
            message=Message(
                role="assistant",
                content=data["content"][0]["text"],
            ),
            usage={
                "prompt_tokens": data["usage"]["input_tokens"],
                "completion_tokens": data["usage"]["output_tokens"],
                "total_tokens": data["usage"]["input_tokens"] + data["usage"]["output_tokens"],
            },
            latency_ms=latency_ms,
        )

    async def chat_stream(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        """Anthropic doesn't support streaming in the same way."""
        # For now, yield a non-streaming response
        response = await self.chat(request)
        yield ChatChunk(delta=response.message.content, finished=True)

    async def embeddings(self, request: EmbeddingsRequest) -> EmbeddingsResponse:
        """Anthropic doesn't have a public embeddings API."""
        raise NotImplementedError("Anthropic does not provide embeddings API")
