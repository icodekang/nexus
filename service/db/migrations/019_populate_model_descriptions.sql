-- Migration: 019_populate_model_descriptions
-- Description: Populate the description field for all seeded models

UPDATE models SET description = 'DeepSeek Chat is a highly efficient conversational model optimized for multi-turn dialogue. It supports vision capabilities for image understanding, function calling for tool integration, and delivers strong performance across reasoning, coding, and content generation tasks. With a 64K context window, it handles long conversations without losing coherence.' WHERE slug = 'deepseek-chat';

UPDATE models SET description = 'DeepSeek Coder is a specialized code generation and analysis model built for software development workflows. It excels at code completion, refactoring, debugging, and generating code across multiple programming languages. With function calling support and a 64K context window, it handles large codebases and complex programming tasks with precision.' WHERE slug = 'deepseek-coder';

UPDATE models SET description = 'GPT-4o is OpenAI''s multimodal flagship model, combining state-of-the-art reasoning with native vision understanding. It offers fast response times, reliable function calling for tool use, and excels at complex analysis, creative writing, and nuanced instruction following. The 128K context window enables processing of entire documents, long conversations, and dense reference materials in a single request.' WHERE slug = 'gpt-4o';

UPDATE models SET description = 'Claude 3.5 Sonnet by Anthropic delivers exceptional reasoning capability with a massive 200K token context window. It excels at nuanced analysis, long-form content creation, code generation, and multi-step problem solving. With vision support and reliable function calling, it is ideal for enterprise workflows, research, and tasks requiring careful attention to detail and safety.' WHERE slug = 'claude-3-5-sonnet';

UPDATE models SET description = 'GPT-4o Mini is OpenAI''s cost-efficient yet powerful model, offering strong reasoning and conversation capabilities at a fraction of the cost of GPT-4o. It supports function calling for tool integration and features a 128K context window. Ideal for high-volume applications, chatbots, and tasks where balancing quality with cost is essential.' WHERE slug = 'gpt-4o-mini';

UPDATE models SET description = 'Gemini 1.5 Pro by Google DeepMind features an industry-leading 1M token context window, capable of processing entire books, video transcripts, or massive codebases in a single prompt. It combines strong multimodal reasoning with native vision understanding and function calling. Ideal for document analysis, long-form content generation, and research applications that demand processing vast amounts of context.' WHERE slug = 'gemini-1-5-pro';
