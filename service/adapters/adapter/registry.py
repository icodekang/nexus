"""
Provider adapter registry.
"""
from typing import Dict, Optional
from .base import BaseAdapter
from .openai import OpenAIAdapter
from .anthropic import AnthropicAdapter
from .google import GoogleAdapter
from .deepseek import DeepSeekAdapter


class AdapterRegistry:
    """Registry for all LLM provider adapters."""

    def __init__(self):
        self._adapters: Dict[str, BaseAdapter] = {}
        self._register_default_adapters()

    def _register_default_adapters(self):
        """Register default adapters."""
        self.register("openai", OpenAIAdapter())
        self.register("anthropic", AnthropicAdapter())
        self.register("google", GoogleAdapter())
        self.register("deepseek", DeepSeekAdapter())

    def register(self, name: str, adapter: BaseAdapter):
        """Register a new adapter."""
        self._adapters[name] = adapter

    def get(self, name: str) -> Optional[BaseAdapter]:
        """Get an adapter by name."""
        return self._adapters.get(name)

    def list_providers(self) -> list:
        """List all registered provider names."""
        return list(self._adapters.keys())

    async def health_check_all(self) -> Dict[str, bool]:
        """Check health of all adapters."""
        results = {}
        for name, adapter in self._adapters.items():
            try:
                results[name] = await adapter.health_check()
            except Exception as e:
                print(f"Health check failed for {name}: {e}")
                results[name] = False
        return results


# Global registry instance
_registry: Optional[AdapterRegistry] = None


def get_registry() -> AdapterRegistry:
    """Get the global adapter registry."""
    global _registry
    if _registry is None:
        _registry = AdapterRegistry()
    return _registry
