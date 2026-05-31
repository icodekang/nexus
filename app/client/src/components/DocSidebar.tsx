import { useState, useCallback, useEffect } from 'react';
import {
  BookOpen,
  ChevronDown,
  PanelLeftClose,
  PanelLeft,
  FileText,
  Mic,
  Image as ImageIcon,
  Film,
  Braces,
  Globe,
  Server,
  Volume2,
  Users,
  Music,
  Palette,
  ImagePlus,
  Clapperboard,
  Video,
  Cpu,
} from 'lucide-react';
import { useI18n } from '../i18n';
import { sidebarState } from '../stores/sidebarStore';
import './DocSidebar.css';

interface DocNavItem {
  key: string;
  label: string;
  icon: React.ReactNode;
}

interface DocSection {
  key: string;
  label: string;
  icon: React.ReactNode;
  items: DocNavItem[];
}

const ITEM_ICONS: Record<string, React.ReactNode> = {
  'anthropic-api': <Cpu size={13} strokeWidth={1.5} />,
  'openai-api': <Braces size={13} strokeWidth={1.5} />,
  'anthropic-http-api': <Globe size={13} strokeWidth={1.5} />,
  'openai-http-api': <Server size={13} strokeWidth={1.5} />,
  'speech-synthesis': <Volume2 size={13} strokeWidth={1.5} />,
  'voice-management': <Users size={13} strokeWidth={1.5} />,
  'music-generation': <Music size={13} strokeWidth={1.5} />,
  'text-to-image': <Palette size={13} strokeWidth={1.5} />,
  'image-to-image': <ImagePlus size={13} strokeWidth={1.5} />,
  'text-to-video': <Video size={13} strokeWidth={1.5} />,
  'image-to-video': <Clapperboard size={13} strokeWidth={1.5} />,
};

export default function DocSidebar() {
  const { t } = useI18n();
  const [collapsed, setCollapsed] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['text']));
  const [activeItem, setActiveItem] = useState<string | null>(null);

  const sections: DocSection[] = [
    {
      key: 'text',
      label: t('docs.sectionText'),
      icon: <FileText size={14} strokeWidth={1.75} />,
      items: [
        { key: 'anthropic-api', label: t('docs.anthropicApi'), icon: ITEM_ICONS['anthropic-api'] },
        { key: 'openai-api', label: t('docs.openaiApi'), icon: ITEM_ICONS['openai-api'] },
        { key: 'anthropic-http-api', label: t('docs.anthropicHttpApi'), icon: ITEM_ICONS['anthropic-http-api'] },
        { key: 'openai-http-api', label: t('docs.openaiHttpApi'), icon: ITEM_ICONS['openai-http-api'] },
      ],
    },
    {
      key: 'voice',
      label: t('docs.sectionVoice'),
      icon: <Mic size={14} strokeWidth={1.75} />,
      items: [
        { key: 'speech-synthesis', label: t('docs.speechSynthesis'), icon: ITEM_ICONS['speech-synthesis'] },
        { key: 'voice-management', label: t('docs.voiceManagement'), icon: ITEM_ICONS['voice-management'] },
        { key: 'music-generation', label: t('docs.musicGeneration'), icon: ITEM_ICONS['music-generation'] },
      ],
    },
    {
      key: 'image',
      label: t('docs.sectionImage'),
      icon: <ImageIcon size={14} strokeWidth={1.75} />,
      items: [
        { key: 'text-to-image', label: t('docs.textToImage'), icon: ITEM_ICONS['text-to-image'] },
        { key: 'image-to-image', label: t('docs.imageToImage'), icon: ITEM_ICONS['image-to-image'] },
      ],
    },
    {
      key: 'video',
      label: t('docs.sectionVideo'),
      icon: <Film size={14} strokeWidth={1.75} />,
      items: [
        { key: 'text-to-video', label: t('docs.textToVideo'), icon: ITEM_ICONS['text-to-video'] },
        { key: 'image-to-video', label: t('docs.imageToVideo'), icon: ITEM_ICONS['image-to-video'] },
      ],
    },
  ];

  const toggleSection = useCallback((key: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  }, []);

  const handleItemClick = useCallback(
    (itemKey: string) => {
      const newActive = activeItem === itemKey ? null : itemKey;
      setActiveItem(newActive);
      sidebarState.setActiveKey(newActive);

      const section = sections.find((s) => s.items.some((i) => i.key === itemKey));
      if (section && !expandedSections.has(section.key)) {
        setExpandedSections((prev) => new Set([...prev, section.key]));
      }
    },
    [activeItem, sections, expandedSections],
  );

  useEffect(() => {
    const unsub = sidebarState.subscribe(() => {
      setActiveItem(sidebarState.getActiveKey());
    });
    return unsub;
  }, []);

  if (collapsed) {
    return (
      <div className="docsidebar-collapsed-strip">
        <button
          className="docsidebar-expand-btn"
          onClick={() => setCollapsed(false)}
          title={t('docs.expandSidebar')}
        >
          <PanelLeft size={15} strokeWidth={1.75} />
        </button>
      </div>
    );
  }

  return (
    <aside className="docsidebar">
      <div className="docsidebar-inner">
        <div className="docsidebar-header">
          <div className="docsidebar-header-content">
            <BookOpen size={15} strokeWidth={1.5} className="docsidebar-header-icon" />
            <span className="docsidebar-header-title">{t('docs.title')}</span>
          </div>
          <button
            className="docsidebar-collapse-btn"
            onClick={() => setCollapsed(true)}
            title={t('docs.collapseSidebar')}
          >
            <PanelLeftClose size={14} strokeWidth={1.5} />
          </button>
        </div>

        <nav className="docsidebar-nav">
          {sections.map((section) => {
            const isExpanded = expandedSections.has(section.key);
            return (
              <div key={section.key} className="docsidebar-section">
                <button
                  className={`docsidebar-section-header ${isExpanded ? 'expanded' : ''}`}
                  onClick={() => toggleSection(section.key)}
                >
                  <span className="docsidebar-section-icon">{section.icon}</span>
                  <span className="docsidebar-section-label">{section.label}</span>
                  <ChevronDown
                    size={12}
                    strokeWidth={1.75}
                    className={`docsidebar-chevron ${isExpanded ? 'rotated' : ''}`}
                  />
                </button>
                <div className={`docsidebar-section-items ${isExpanded ? 'expanded' : ''}`}>
                  <div className="docsidebar-section-items-inner">
                    {section.items.map((item) => (
                      <button
                        key={item.key}
                        className={`docsidebar-item ${activeItem === item.key ? 'active' : ''}`}
                        onClick={() => handleItemClick(item.key)}
                      >
                        <span className="docsidebar-item-icon">{item.icon}</span>
                        <span className="docsidebar-item-label">{item.label}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            );
          })}
        </nav>

        <div className="docsidebar-footer">
          <span className="docsidebar-footer-dot" />
          <span className="docsidebar-footer-text">Nexus API Reference</span>
        </div>
      </div>
    </aside>
  );
}
