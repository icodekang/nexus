"""
Google AI (Gemini) adapter.
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


class GoogleAdapter(BaseAdapter):
    """Adapter for Google AI Gemini API."""

    BASE_URL = "https://generativelanguage.googleapis.com/v1beta"

    def __init__(self):
        self.api_key = os.getenv("GOOGLE_API_KEY", "")
        self.client = httpx.AsyncClient(timeout=60.0)

    @property
    def provider_name(self) -> str:
        return "google"

    def _convert_to_gemini_format(self, messages):
        """Convert messages to Gemini format."""
        contents = []
        for msg in messages:
            if msg.role == "user":
                contents.append({
                    "role": "user",
                    "parts": [{"text": msg.content}]
                })
            elif msg.role == "assistant":
                contents.append({
                    "role": "model",
                    "parts": [{"text": msg.content}]
                })
        return contents

    async def chat(self, request: ChatRequest) -> ChatResponse:
        """Send a chat request to Google Gemini."""
        url = f"{self.BASE_URL}/models/{request.model}:generateContent"

        params = {"key": self.api_key}
        headers = {"Content-Type": "application/json"}

        payload = {
            "contents": self._convert_to_gemini_format(request.messages),
            "generationConfig": {
                "temperature": request.temperature,
                "maxOutputTokens": request.max_tokens or 2048,
            }
        }

        start = time.time()
        response = await self.client.post(url, json=payload, headers=headers, params=params)
        latency_ms = int((time.time() - start) * 1000)

        response.raise_for_status()
        data = response.json()

        text = data["candidates"][0]["content"]["parts"][0]["text"]

        return ChatResponse(
            id=f"gemini_{int(time.time())}",
            model=request.model,
            message=Message(role="assistant", content=text),
            usage={
                "prompt_tokens": 0,  # Google doesn't expose this
                "completion_tokens": 0,
                "total_tokens": 0,
            },
            latency_ms=latency_ms,
        )

    async def chat_stream(self, request: ChatRequest) -> AsyncIterator[ChatChunk]:
        """Send a streaming chat request to Google Gemini."""
        url = f"{self.BASE_URL}/models/{request.model}:streamGenerateContent"

        params = {"key": self.api_key}
        headers = {"Content-Type": "application/json"}

        payload = {
            "contents": self._convert_to_gemini_format(request.messages),
            "generationConfig": {
                "temperature": request.temperature,
                "maxOutputTokens": request.max_tokens or 2048,
            }
        }

        async with self.client.stream("POST", url, json=payload, headers=headers, params=params) as response:
            response.raise_for_status()
            async for line in response.aiter_lines():
                if line:
                    try:
                        data = json.loads(line)
                        if "candidates" in data:
                            text = data["candidates"][0]["content"]["parts"][0]["text"]
                            yield ChatChunk(delta=text, finished=False)
                    except json.JSONDecodeError:
                        pass
            yield ChatChunk(delta="", finished=True)

    async def embeddings(self, request: EmbeddingsRequest) -> EmbeddingsResponse:
        """Generate embeddings using Google."""
        url = f"{self.BASE_URL}/models/embedding-001:embedContent"

        params = {"key": self.api_key}
        headers = {"Content-Type": "application/json"}

        embeddings = []
        for i, text in enumerate(request.inputs):
            payload = {
                "model": f"models/{request.model}",
                "content": {"parts": [{"text": text}]}
            }
            response = await self.client.post(url, json=payload, headers=headers, params=params)
            response.raise_for_status()
            data = response.json()
            embeddings.append(data["embedding"]["values"])

        return EmbeddingsResponse(embeddings=embeddings)
