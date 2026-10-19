import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';
import ptBR from './locales/pt-BR.json';
import enUS from './locales/en-US.json';
import esES from './locales/es-ES.json';
import frFR from './locales/fr-FR.json';

export const LANGUAGE_STORAGE_KEY = 'astral_language';
export const supportedLanguages = [
    { code: 'pt-BR', label: 'Português' },
    { code: 'en-US', label: 'English' },
    { code: 'es-ES', label: 'Español' },
    { code: 'fr-FR', label: 'Français' }
];

void i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources: {
            'pt-BR': { translation: ptBR },
            'en-US': { translation: enUS },
            'es-ES': { translation: esES },
            'fr-FR': { translation: frFR }
        },
        fallbackLng: 'pt-BR',
        supportedLngs: supportedLanguages.map(({ code }) => code),
        load: 'currentOnly',
        ns: ['translation'],
        defaultNS: 'translation',
        interpolation: { escapeValue: false },
        detection: {
            order: ['localStorage', 'navigator'],
            lookupLocalStorage: LANGUAGE_STORAGE_KEY,
            caches: ['localStorage']
        }
    });

export default i18n;

export const syncDocumentLanguage = (language?: string) => {
    if (typeof document === 'undefined') return;
    const code = supportedLanguages.some(item => item.code === language)
        ? (language as string)
        : 'pt-BR';
    document.documentElement.lang = code;
};

syncDocumentLanguage(i18n.resolvedLanguage || i18n.language);
i18n.on('languageChanged', syncDocumentLanguage);