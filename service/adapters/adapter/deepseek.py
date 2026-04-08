"""
DeepSeek API adapter.
"""
import os
import time
import json
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


class DeepSeekAdapter(BaseAdapter):
    """Adapter for DeepSeek API."""

    BASE_URL = "https://api.deepseek.com/v1"

    def __init__(self):
        self.api_key = os.getenv("DEEPSEEK_API_KEY", "")
        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY environment variable is not set")
        self.client = httpx.AsyncClient(timeout=60.0)

    @property
    def provider_name(self) -> str:
        return "deepseek"

    async def chat(self, request: ChatRequest) -> ChatResponse:
        """Send a chat request to DeepSeek."""
        url = f"{self.BASE_URL}/chat/completions"

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": request.model,
            "messages": [m.model_dump() for m in request.messages],
            "temperature": request.temperature,
            "stream": False,
        }

        if request.max_tokens:
            payload["max_tokens"] = request.max_tokens

        start = time.time()
        response = await self.client.post(url, json=payload, headers=headers)
        latency_ms = int((time.time() - start) * 1000)

        response.raise_for_status()
        data = response.json()

        return ChatResponse(
            id=data["id"],
            model=data["model"],
            message=Message(
                role=data["choices"][0]["message"]["role"],
                content=data["choices"][0]["message"]["content"],
            ),
            usage=data.get("usage", {}),
            latency_ms=latency_ms,
        )

    async def chat_stream(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        """Send a streaming chat request to DeepSeek."""
        url = f"{self.BASE_URL}/chat/completions"

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": request.model,
            "messages": [m.model_dump() for m in request.messages],
            "temperature": request.temperature,
            "stream": True,
        }

        if request.max_tokens:
            payload["max_tokens"] = request.max_tokens

        async with self.client.stream("POST", url, json=payload, headers=headers) as response:
            response.raise_for_status()
            async for line in response.aiter_lines():
                if line.startswith("data: "):
                    data = line[6:]
                    if data == "[DONE]":
                        yield ChatChunk(delta="", finished=True)
                        break
                    chunk_data = json.loads(data)
                    delta = chunk_data["choices"][0]["delta"].get("content", "")
                    yield ChatChunk(delta=delta, finished=False)

    async def embeddings(self, request: EmbeddingsRequest) -> EmbeddingsResponse:
        """Generate embeddings using DeepSeek."""
        url = f"{self.BASE_URL}/embeddings"

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": request.model,
            "input": request.inputs,
        }

        response = await self.client.post(url, json=payload, headers=headers)
        response.raise_for_status()
        data = response.json()

        return EmbeddingsResponse(
            embeddings=[item["embedding"] for item in data["data"]]
        )
