use std::sync::atomic::{AtomicU8, Ordering};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    De,
    Es,
    Fr,
    Zh,
}

impl Lang {
    const ALL: [Lang; 5] = [Lang::En, Lang::De, Lang::Es, Lang::Fr, Lang::Zh];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::Zh => "zh",
        }
    }

    fn from_locale(locale: &str) -> Option<Lang> {
        let prefix = locale.split(['-', '_']).next()?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == prefix)
    }

    /// The explicit choice, else the system language if supported, else English.
    pub fn resolve(setting: Option<Lang>) -> Lang {
        setting
            .or_else(|| sys_locale::get_locale().and_then(|l| Lang::from_locale(&l)))
            .unwrap_or(Lang::En)
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_language(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

pub fn language() -> Lang {
    Lang::ALL[usize::from(CURRENT.load(Ordering::Relaxed)).min(Lang::ALL.len() - 1)]
}

pub struct Strings {
    pub not_connected: &'static str,
    pub checking: &'static str,
    pub possibly_down: &'static str,
    pub invalid_key: &'static str,
    pub cannot_reach: &'static str,
    pub all_up: &'static str,
    pub checked: &'static str,
    pub muted_until: &'static str,
    pub mute: &'static str,
    pub minutes_15: &'static str,
    pub minutes_30: &'static str,
    pub hour_1: &'static str,
    pub unmute: &'static str,
    pub open_dashboard: &'static str,
    pub check_now: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
    pub setup_title: &'static str,
    pub settings_title: &'static str,
    pub about: &'static str,
    /// `{n}` is replaced by the number of sites.
    one_site_down: &'static str,
    sites_down: &'static str,
    units: (&'static str, &'static str, &'static str),
}

const EN: Strings = Strings {
    not_connected: "Not connected",
    checking: "Checking …",
    possibly_down: "Possibly down",
    invalid_key: "Invalid API key",
    cannot_reach: "Cannot reach UptimeRobot",
    all_up: "All up",
    checked: "checked",
    muted_until: "Muted until",
    mute: "Mute",
    minutes_15: "15 minutes",
    minutes_30: "30 minutes",
    hour_1: "1 hour",
    unmute: "Unmute",
    open_dashboard: "Open UptimeRobot dashboard",
    check_now: "Check now",
    settings: "Settings …",
    quit: "Quit UptimeWidget",
    setup_title: "Set up UptimeWidget",
    settings_title: "UptimeWidget – Settings",
    about: "About UptimeWidget",
    one_site_down: "1 site down",
    sites_down: "{n} sites down",
    units: ("d", "h", "m"),
};

const DE: Strings = Strings {
    not_connected: "Nicht verbunden",
    checking: "Wird geprüft …",
    possibly_down: "Möglicherweise offline",
    invalid_key: "API-Key ungültig",
    cannot_reach: "Keine Verbindung zu UptimeRobot",
    all_up: "Alles online",
    checked: "geprüft",
    muted_until: "Stumm bis",
    mute: "Stummschalten",
    minutes_15: "15 Minuten",
    minutes_30: "30 Minuten",
    hour_1: "1 Stunde",
    unmute: "Stummschaltung aufheben",
    open_dashboard: "UptimeRobot-Dashboard öffnen",
    check_now: "Jetzt prüfen",
    settings: "Einstellungen …",
    quit: "UptimeWidget beenden",
    setup_title: "UptimeWidget einrichten",
    settings_title: "UptimeWidget – Einstellungen",
    about: "Über UptimeWidget",
    one_site_down: "1 Site offline",
    sites_down: "{n} Sites offline",
    units: (" T", " Std", " Min"),
};

const ES: Strings = Strings {
    not_connected: "No conectado",
    checking: "Comprobando …",
    possibly_down: "Posiblemente caído",
    invalid_key: "Clave de API no válida",
    cannot_reach: "No se puede conectar con UptimeRobot",
    all_up: "Todo en línea",
    checked: "comprobado",
    muted_until: "Silenciado hasta",
    mute: "Silenciar",
    minutes_15: "15 minutos",
    minutes_30: "30 minutos",
    hour_1: "1 hora",
    unmute: "Quitar silencio",
    open_dashboard: "Abrir panel de UptimeRobot",
    check_now: "Comprobar ahora",
    settings: "Ajustes …",
    quit: "Salir de UptimeWidget",
    setup_title: "Configurar UptimeWidget",
    settings_title: "UptimeWidget – Ajustes",
    about: "Acerca de UptimeWidget",
    one_site_down: "1 sitio caído",
    sites_down: "{n} sitios caídos",
    units: (" d", " h", " min"),
};

const FR: Strings = Strings {
    not_connected: "Non connecté",
    checking: "Vérification …",
    possibly_down: "Peut-être hors ligne",
    invalid_key: "Clé API invalide",
    cannot_reach: "Impossible de joindre UptimeRobot",
    all_up: "Tout est en ligne",
    checked: "vérifié",
    muted_until: "En sourdine jusqu’à",
    mute: "Mettre en sourdine",
    minutes_15: "15 minutes",
    minutes_30: "30 minutes",
    hour_1: "1 heure",
    unmute: "Réactiver",
    open_dashboard: "Ouvrir le tableau de bord UptimeRobot",
    check_now: "Vérifier maintenant",
    settings: "Réglages …",
    quit: "Quitter UptimeWidget",
    setup_title: "Configurer UptimeWidget",
    settings_title: "UptimeWidget – Réglages",
    about: "À propos d’UptimeWidget",
    one_site_down: "1 site hors ligne",
    sites_down: "{n} sites hors ligne",
    units: (" j", " h", " min"),
};

const ZH: Strings = Strings {
    not_connected: "未连接",
    checking: "正在检查 …",
    possibly_down: "可能已离线",
    invalid_key: "API 密钥无效",
    cannot_reach: "无法连接 UptimeRobot",
    all_up: "全部在线",
    checked: "检查于",
    muted_until: "静音至",
    mute: "静音",
    minutes_15: "15 分钟",
    minutes_30: "30 分钟",
    hour_1: "1 小时",
    unmute: "取消静音",
    open_dashboard: "打开 UptimeRobot 控制台",
    check_now: "立即检查",
    settings: "设置 …",
    quit: "退出 UptimeWidget",
    setup_title: "设置 UptimeWidget",
    settings_title: "UptimeWidget – 设置",
    about: "关于 UptimeWidget",
    one_site_down: "1 个网站离线",
    sites_down: "{n} 个网站离线",
    units: (" 天", " 小时", " 分钟"),
};

pub fn s() -> &'static Strings {
    match language() {
        Lang::En => &EN,
        Lang::De => &DE,
        Lang::Es => &ES,
        Lang::Fr => &FR,
        Lang::Zh => &ZH,
    }
}

pub fn sites_offline(count: usize) -> String {
    let s = s();
    if count == 1 {
        s.one_site_down.into()
    } else {
        s.sites_down.replace("{n}", &count.to_string())
    }
}

/// Compact duration like "13d 16h" / "13 T 16 Std".
pub fn duration_since(since: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let minutes = (now - since).num_minutes().max(0);
    let (days, hours, mins) = (minutes / 1440, minutes / 60 % 24, minutes % 60);
    let (d, h, m) = s().units;
    if days > 0 {
        format!("{days}{d} {hours}{h}")
    } else if hours > 0 {
        format!("{hours}{h} {mins}{m}")
    } else {
        format!("{mins}{m}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_locales() {
        assert_eq!(Lang::from_locale("de-CH"), Some(Lang::De));
        assert_eq!(Lang::from_locale("zh_Hans_CN"), Some(Lang::Zh));
        assert_eq!(Lang::from_locale("ja-JP"), None);
        assert_eq!(Lang::resolve(Some(Lang::Fr)), Lang::Fr);
    }
}
