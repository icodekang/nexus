import { useState, useEffect } from 'react';
import {
  Cpu,
  Braces,
  Globe,
  Server,
  Volume2,
  Users,
  Music,
  Palette,
  ImagePlus,
  Video,
  Clapperboard,
  ArrowRight,
  BookOpen,
  Zap,
  AlertTriangle,
  Terminal,
} from 'lucide-react';
import { useI18n } from '../i18n';
import { sidebarState } from '../stores/sidebarStore';
import './GuidePage.css';

interface ParamDef {
  param: string;
  type: string;
  required: string;
  desc: string;
}

interface TopicContent {
  icon: React.ReactNode;
  title: string;
  desc: string;
  method: string;
  path: string;
  endpointDesc: string;
  params: ParamDef[];
  curl: string;
  curlDesc?: string;
  sdkExample?: { lang: string; code: string; note: string };
  noteTag: string;
  noteText: string;
}

function getTopicContent(key: string, t: (k: string) => string): TopicContent | null {
  const iconMap: Record<string, React.ReactNode> = {
    'anthropic-api': <Cpu size={16} strokeWidth={1.75} />,
    'openai-api': <Braces size={16} strokeWidth={1.75} />,
    'anthropic-http-api': <Globe size={16} strokeWidth={1.75} />,
    'openai-http-api': <Server size={16} strokeWidth={1.75} />,
    'speech-synthesis': <Volume2 size={16} strokeWidth={1.75} />,
    'voice-management': <Users size={16} strokeWidth={1.75} />,
    'music-generation': <Music size={16} strokeWidth={1.75} />,
    'text-to-image': <Palette size={16} strokeWidth={1.75} />,
    'image-to-image': <ImagePlus size={16} strokeWidth={1.75} />,
    'text-to-video': <Video size={16} strokeWidth={1.75} />,
    'image-to-video': <Clapperboard size={16} strokeWidth={1.75} />,
  };

  const map: Record<string, TopicContent> = {
    'anthropic-api': {
      icon: iconMap['anthropic-api'],
      title: t('docsContent.anthropicApiTitle'),
      desc: t('docsContent.anthropicApiDesc'),
      method: 'POST',
      path: '/v1/messages',
      endpointDesc: t('docsContent.anthropicApiEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.anthropicApiParamModel') },
        { param: 'messages', type: 'array', required: 'Yes', desc: t('docsContent.anthropicApiParamMessages') },
        { param: 'max_tokens', type: 'integer', required: 'Yes', desc: t('docsContent.anthropicApiParamMaxTokens') },
        { param: 'stream', type: 'boolean', required: 'No', desc: t('docsContent.anthropicApiParamStream') },
        { param: 'temperature', type: 'number', required: 'No', desc: t('docsContent.anthropicApiParamTemperature') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/messages \\',
        '  -H "x-api-key: sk-nexus-your-key" \\',
        '  -H "anthropic-version: 2023-06-01" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "claude-sonnet-4-20250514",',
        '    "max_tokens": 1024,',
        '    "messages": [',
        '      {"role": "user", "content": "Explain quantum computing"}',
        '    ]',
        "  }'",
      ].join('\n'),
      sdkExample: {
        lang: 'python',
        code: [
          'import anthropic',
          '',
          'client = anthropic.Anthropic(',
          '    api_key="sk-nexus-your-key",',
          '    base_url="https://your-nexus.com/v1"',
          ')',
          '',
          'response = client.messages.create(',
          '    model="claude-sonnet-4-20250514",',
          '    max_tokens=1024,',
          '    messages=[',
          '        {"role": "user", "content": "Explain quantum computing"}',
          '    ]',
          ')',
          '',
          'print(response.content)',
        ].join('\n'),
        note: t('docsContent.anthropicApiSdkNote'),
      },
      noteTag: t('docsContent.anthropicApiNoteTag'),
      noteText: t('docsContent.anthropicApiNoteText'),
    },
    'openai-api': {
      icon: iconMap['openai-api'],
      title: t('docsContent.openaiApiTitle'),
      desc: t('docsContent.openaiApiDesc'),
      method: 'POST',
      path: '/v1/chat/completions',
      endpointDesc: t('docsContent.openaiApiEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.openaiApiParamModel') },
        { param: 'messages', type: 'array', required: 'Yes', desc: t('docsContent.openaiApiParamMessages') },
        { param: 'stream', type: 'boolean', required: 'No', desc: t('docsContent.openaiApiParamStream') },
        { param: 'temperature', type: 'number', required: 'No', desc: t('docsContent.openaiApiParamTemperature') },
        { param: 'tools', type: 'array', required: 'No', desc: t('docsContent.openaiApiParamTools') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/chat/completions \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "gpt-4o",',
        '    "messages": [',
        '      {"role": "user", "content": "Hello!"}',
        '    ]',
        "  }'",
      ].join('\n'),
      sdkExample: {
        lang: 'python',
        code: [
          'from openai import OpenAI',
          '',
          'client = OpenAI(',
          '    api_key="sk-nexus-your-key",',
          '    base_url="https://your-nexus.com/v1"',
          ')',
          '',
          'response = client.chat.completions.create(',
          '    model="gpt-4o",',
          '    messages=[',
          '        {"role": "user", "content": "Hello!"}',
          '    ]',
          ')',
          '',
          'print(response.choices[0].message.content)',
        ].join('\n'),
        note: t('docsContent.openaiApiSdkNote'),
      },
      noteTag: t('docsContent.openaiApiNoteTag'),
      noteText: t('docsContent.openaiApiNoteText'),
    },
    'anthropic-http-api': {
      icon: iconMap['anthropic-http-api'],
      title: t('docsContent.anthropicHttpTitle'),
      desc: t('docsContent.anthropicHttpDesc'),
      method: 'POST',
      path: '/v1/messages',
      endpointDesc: t('docsContent.anthropicHttpEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.anthropicApiParamModel') },
        { param: 'messages', type: 'array', required: 'Yes', desc: t('docsContent.anthropicApiParamMessages') },
        { param: 'max_tokens', type: 'integer', required: 'Yes', desc: t('docsContent.anthropicApiParamMaxTokens') },
        { param: 'stream', type: 'boolean', required: 'No', desc: t('docsContent.anthropicApiParamStream') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/messages \\',
        '  -H "x-api-key: sk-nexus-your-key" \\',
        '  -H "anthropic-version: 2023-06-01" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "claude-sonnet-4-20250514",',
        '    "max_tokens": 1024,',
        '    "messages": [{"role": "user", "content": "Hello"}]',
        "  }'",
      ].join('\n'),
      noteTag: t('docsContent.anthropicApiNoteTag'),
      noteText: t('docsContent.anthropicHttpRespNote'),
    },
    'openai-http-api': {
      icon: iconMap['openai-http-api'],
      title: t('docsContent.openaiHttpTitle'),
      desc: t('docsContent.openaiHttpDesc'),
      method: 'POST',
      path: '/v1/chat/completions',
      endpointDesc: t('docsContent.openaiHttpEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.openaiApiParamModel') },
        { param: 'messages', type: 'array', required: 'Yes', desc: t('docsContent.openaiApiParamMessages') },
        { param: 'stream', type: 'boolean', required: 'No', desc: t('docsContent.openaiApiParamStream') },
        { param: 'temperature', type: 'number', required: 'No', desc: t('docsContent.openaiApiParamTemperature') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/chat/completions \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "gpt-4o",',
        '    "messages": [{"role": "user", "content": "Hello"}]',
        "  }'",
      ].join('\n'),
      noteTag: t('docsContent.openaiApiNoteTag'),
      noteText: t('docsContent.openaiHttpRespNote'),
    },
    'speech-synthesis': {
      icon: iconMap['speech-synthesis'],
      title: t('docsContent.speechTitle'),
      desc: t('docsContent.speechDesc'),
      method: 'POST',
      path: '/v1/audio/speech',
      endpointDesc: t('docsContent.speechEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.speechParamModel') },
        { param: 'input', type: 'string', required: 'Yes', desc: t('docsContent.speechParamInput') },
        { param: 'voice', type: 'string', required: 'Yes', desc: t('docsContent.speechParamVoice') },
        { param: 'response_format', type: 'string', required: 'No', desc: t('docsContent.speechParamFormat') },
        { param: 'speed', type: 'number', required: 'No', desc: t('docsContent.speechParamSpeed') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/audio/speech \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "tts-1",',
        '    "input": "Hello, welcome to Nexus.",',
        '    "voice": "alloy",',
        '    "response_format": "mp3"',
        "  }' \\",
        '  --output speech.mp3',
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.speechNoteText'),
    },
    'voice-management': {
      icon: iconMap['voice-management'],
      title: t('docsContent.voiceManagementTitle'),
      desc: t('docsContent.voiceManagementDesc'),
      method: 'GET',
      path: '/v1/voices',
      endpointDesc: t('docsContent.voiceMgmtListDesc'),
      params: [
        { param: 'model', type: 'string', required: 'No', desc: t('docsContent.voiceMgmtParamModel') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/voices \\',
        '  -H "Authorization: Bearer sk-nexus-your-key"',
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.voiceMgmtNoteText'),
    },
    'music-generation': {
      icon: iconMap['music-generation'],
      title: t('docsContent.musicTitle'),
      desc: t('docsContent.musicDesc'),
      method: 'POST',
      path: '/v1/music/generations',
      endpointDesc: t('docsContent.musicEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.musicParamModel') },
        { param: 'prompt', type: 'string', required: 'Yes', desc: t('docsContent.musicParamPrompt') },
        { param: 'lyrics', type: 'string', required: 'No', desc: t('docsContent.musicParamLyrics') },
        { param: 'duration', type: 'integer', required: 'No', desc: t('docsContent.musicParamDuration') },
        { param: 'instrumental', type: 'boolean', required: 'No', desc: t('docsContent.musicParamInstrumental') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/music/generations \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "music-1",',
        '    "prompt": "An upbeat pop song with synth",',
        '    "lyrics": "[Verse]\\nWalking through the city...",',
        '    "duration": 180',
        "  }'",
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.musicNoteText'),
    },
    'text-to-image': {
      icon: iconMap['text-to-image'],
      title: t('docsContent.textToImageTitle'),
      desc: t('docsContent.textToImageDesc'),
      method: 'POST',
      path: '/v1/images/generations',
      endpointDesc: t('docsContent.textToImageEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.t2iParamModel') },
        { param: 'prompt', type: 'string', required: 'Yes', desc: t('docsContent.t2iParamPrompt') },
        { param: 'n', type: 'integer', required: 'No', desc: t('docsContent.t2iParamN') },
        { param: 'size', type: 'string', required: 'No', desc: t('docsContent.t2iParamSize') },
        { param: 'quality', type: 'string', required: 'No', desc: t('docsContent.t2iParamQuality') },
        { param: 'style', type: 'string', required: 'No', desc: t('docsContent.t2iParamStyle') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/images/generations \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "dall-e-3",',
        '    "prompt": "A serene mountain lake at sunset",',
        '    "n": 1,',
        '    "size": "1024x1024"',
        "  }'",
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.t2iNoteText'),
    },
    'image-to-image': {
      icon: iconMap['image-to-image'],
      title: t('docsContent.imageToImageTitle'),
      desc: t('docsContent.imageToImageDesc'),
      method: 'POST',
      path: '/v1/images/edits',
      endpointDesc: t('docsContent.imageToImageEndpointDesc'),
      params: [
        { param: 'image', type: 'file', required: 'Yes', desc: t('docsContent.i2iParamImage') },
        { param: 'prompt', type: 'string', required: 'Yes', desc: t('docsContent.i2iParamPrompt') },
        { param: 'mask', type: 'file', required: 'No', desc: t('docsContent.i2iParamMask') },
        { param: 'n', type: 'integer', required: 'No', desc: t('docsContent.i2iParamN') },
        { param: 'size', type: 'string', required: 'No', desc: t('docsContent.i2iParamSize') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/images/edits \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -F "image=@source.png" \\',
        '  -F "prompt=A watercolor painting style" \\',
        '  -F "n=1" \\',
        '  -F "size=1024x1024"',
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.i2iNoteText'),
    },
    'text-to-video': {
      icon: iconMap['text-to-video'],
      title: t('docsContent.textToVideoTitle'),
      desc: t('docsContent.textToVideoDesc'),
      method: 'POST',
      path: '/v1/video/generations',
      endpointDesc: t('docsContent.textToVideoEndpointDesc'),
      params: [
        { param: 'model', type: 'string', required: 'Yes', desc: t('docsContent.t2vParamModel') },
        { param: 'prompt', type: 'string', required: 'Yes', desc: t('docsContent.t2vParamPrompt') },
        { param: 'duration', type: 'integer', required: 'No', desc: t('docsContent.t2vParamDuration') },
        { param: 'resolution', type: 'string', required: 'No', desc: t('docsContent.t2vParamResolution') },
        { param: 'fps', type: 'integer', required: 'No', desc: t('docsContent.t2vParamFps') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/video/generations \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -H "Content-Type: application/json" \\',
        "  -d '{",
        '    "model": "video-1",',
        '    "prompt": "A drone flying over a misty forest",',
        '    "duration": 5,',
        '    "resolution": "1080p"',
        "  }'",
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.t2vNoteText'),
    },
    'image-to-video': {
      icon: iconMap['image-to-video'],
      title: t('docsContent.imageToVideoTitle'),
      desc: t('docsContent.imageToVideoDesc'),
      method: 'POST',
      path: '/v1/video/animations',
      endpointDesc: t('docsContent.imageToVideoEndpointDesc'),
      params: [
        { param: 'image', type: 'file', required: 'Yes', desc: t('docsContent.i2vParamImage') },
        { param: 'prompt', type: 'string', required: 'No', desc: t('docsContent.i2vParamPrompt') },
        { param: 'duration', type: 'integer', required: 'No', desc: t('docsContent.i2vParamDuration') },
        { param: 'resolution', type: 'string', required: 'No', desc: t('docsContent.i2vParamResolution') },
        { param: 'motion', type: 'integer', required: 'No', desc: t('docsContent.i2vParamMotion') },
      ],
      curl: [
        'curl https://your-nexus.com/v1/video/animations \\',
        '  -H "Authorization: Bearer sk-nexus-your-key" \\',
        '  -F "image=@photo.jpg" \\',
        '  -F "prompt=Gentle pan across the landscape" \\',
        '  -F "duration=5" \\',
        '  -F "resolution=1080p"',
      ].join('\n'),
      noteTag: t('docsContent.notes'),
      noteText: t('docsContent.i2vNoteText'),
    },
  };

  return map[key] || null;
}

export default function GuidePage() {
  const { t } = useI18n();
  const [activeKey, setActiveKey] = useState<string | null>(sidebarState.getActiveKey());

  useEffect(() => {
    const unsub = sidebarState.subscribe(() => {
      setActiveKey(sidebarState.getActiveKey());
    });
    return unsub;
  }, []);

  const content = activeKey ? getTopicContent(activeKey, t) : null;

  if (!content) {
    return (
      <div className="guidepage">
        <div className="guidepage-empty">
          <div className="guidepage-empty-icon">
            <BookOpen size={32} strokeWidth={1.25} />
          </div>
          <h2 className="guidepage-empty-title">API Reference</h2>
          <p className="guidepage-empty-desc">{t('docsContent.selectTopic')}</p>
          <div className="guidepage-empty-hint">
            <ArrowRight size={13} strokeWidth={1.75} />
            <span>Browse topics in the documentation sidebar</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="guidepage">
      <div className="guidepage-inner">
        {/* Header */}
        <header className="guidepage-header">
          <div className="guidepage-header-badge">
            <span className="guidepage-header-icon">{content.icon}</span>
            <span className="guidepage-header-crumb">{content.title}</span>
          </div>
          <h1 className="guidepage-title">{content.title}</h1>
          <p className="guidepage-desc">{content.desc}</p>
        </header>

        {/* Endpoint Section */}
        <section className="guidepage-section">
          <h2 className="guidepage-section-title">
            <Zap size={14} strokeWidth={1.75} />
            {t('docsContent.endpoints')}
          </h2>
          <div className="guidepage-endpoint-card">
            <span className={`guidepage-method-badge ${content.method.toLowerCase()}`}>
              {content.method}
            </span>
            <code className="guidepage-path">{content.path}</code>
            <span className="guidepage-endpoint-desc">{content.endpointDesc}</span>
          </div>
        </section>

        {/* Request Parameters */}
        <section className="guidepage-section">
          <h2 className="guidepage-section-title">
            <Terminal size={14} strokeWidth={1.75} />
            {t('docsContent.requestParams')}
          </h2>
          <div className="guidepage-table-wrap">
            <table className="guidepage-table">
              <thead>
                <tr>
                  <th>{t('docsContent.parameter')}</th>
                  <th>{t('docsContent.type')}</th>
                  <th>{t('docsContent.required')}</th>
                  <th>{t('docsContent.paramDesc')}</th>
                </tr>
              </thead>
              <tbody>
                {content.params.map((p) => (
                  <tr key={p.param}>
                    <td className="guidepage-param-name">{p.param}</td>
                    <td className="guidepage-param-type">{p.type}</td>
                    <td className="guidepage-param-req">{p.required}</td>
                    <td className="guidepage-param-desc">{p.desc}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Code Example */}
        <section className="guidepage-section">
          <h2 className="guidepage-section-title">
            <ArrowRight size={14} strokeWidth={1.75} />
            {t('docsContent.codeExample')}
          </h2>
          <div className="guidepage-code-label">cURL</div>
          <pre className="guidepage-code">
            <code>{content.curl}</code>
          </pre>
        </section>

        {/* SDK Example */}
        {content.sdkExample && (
          <section className="guidepage-section">
            <h2 className="guidepage-section-title">
              <Braces size={14} strokeWidth={1.75} />
              SDK
            </h2>
            <div className="guidepage-code-label">{content.sdkExample.lang}</div>
            <pre className="guidepage-code">
              <code>{content.sdkExample.code}</code>
            </pre>
            <p className="guidepage-sdk-note">{content.sdkExample.note}</p>
          </section>
        )}

        {/* Notes */}
        <section className="guidepage-section">
          <div className="guidepage-note">
            <div className="guidepage-note-header">
              <AlertTriangle size={14} strokeWidth={1.75} />
              <span>{content.noteTag}</span>
            </div>
            <p className="guidepage-note-text">{content.noteText}</p>
          </div>
        </section>
      </div>
    </div>
  );
}
