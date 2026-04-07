"""
NovaChat Adapter Service
FastAPI application that handles communication with external LLM providers.
"""
import os
from typing import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from adapter.base import (
    ChatRequest,
    ChatResponse,
    ChatChunk,
    EmbeddingsRequest,
    EmbeddingsResponse,
)
from adapter.registry import get_registry


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    print("Starting NovaChat Adapter Service...")
    registry = get_registry()
    print(f"Registered providers: {registry.list_providers()}")
    yield
    # Shutdown
    print("Shutting down...")


app = FastAPI(
    title="NovaChat Adapter Service",
    description="Unified API for multiple LLM providers",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    registry = get_registry()
    health = registry.health_check_all()
    return {
        "status": "healthy",
        "providers": health,
    }


@app.get("/providers")
async def list_providers():
    """List all registered providers."""
    registry = get_registry()
    return {
        "providers": registry.list_providers(),
    }


@app.post("/v1/chat/completions")
async def chat_completions(request: ChatRequest) -> ChatResponse:
    """Handle chat completion requests."""
    registry = get_registry()
    adapter = registry.get(request.provider)

    if not adapter:
        raise HTTPException(status_code=404, detail=f"Provider '{request.provider}' not found")

    try:
        if request.stream:
            # For streaming, we return a generator
            # This is handled separately in the streaming endpoint
            raise HTTPException(status_code=400, detail="Use /v1/chat/stream for streaming")
        return await adapter.chat(request)
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Provider error: {str(e)}")


@app.post("/v1/chat/stream")
async def chat_stream(request: ChatRequest):
    """Handle streaming chat completion requests."""
    registry = get_registry()
    adapter = registry.get(request.provider)

    if not adapter:
        raise HTTPException(status_code=404, detail=f"Provider '{request.provider}' not found")

    if not request.stream:
        request.stream = True

    async def event_generator():
        try:
            async for chunk in adapter.chat_stream(request):
                yield f"data: {chunk.model_dump_json()}\n\n"
            yield "data: [DONE]\n\n"
        except Exception as e:
            yield f"data: {{\"error\": \"{str(e)}\"}}\n\n"

    from fastapi.responses import StreamingResponse
    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
        }
    )


@app.post("/v1/embeddings")
async def embeddings(request: EmbeddingsRequest) -> EmbeddingsResponse:
    """Handle embeddings requests."""
    registry = get_registry()
    adapter = registry.get(request.model)

    if not adapter:
        raise HTTPException(status_code=404, detail=f"Model '{request.model}' not found")

    try:
        return await adapter.embeddings(request)
    except NotImplementedError:
        raise HTTPException(status_code=501, detail="Embeddings not supported for this provider")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Provider error: {str(e)}")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=50051)
