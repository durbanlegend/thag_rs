use rust_i18n::Backend;

pub struct ThagI18n {
    trs: HashMap<String, HashMap<String, String>>,
}

impl ThagI18n {
    fn new() -> Self {
        // Fetch translations from central location
        // let response = reqwest::blocking::get("https://your-host.com/assets/locales.yml").unwrap();
        let response = reqwest::blocking::get("https://your-host.com/assets/locales.yml").unwrap();
        let trs = serde_yaml::from_str::<HashMap<String, HashMap<String, String>>>(
            &response.text().unwrap(),
        )
        .unwrap();

        return Self { trs };
    }
}

impl Backend for ThagI18n {
    fn available_locales(&self) -> Vec<Cow<'_, str>> {
        return self.trs.keys().map(|k| Cow::from(k.as_str())).collect();
    }

    fn translate(&self, locale: &str, key: &str) -> Option<Cow<'_, str>> {
        // Write your own lookup logic here.
        // For example load from database
        return self
            .trs
            .get(locale)?
            .get(key)
            .map(|k| Cow::from(k.as_str()));
    }

    fn messages_for_locale(&self, locale: &str) -> Option<Vec<(Cow<'_, str>, Cow<'_, str>)>> {
        None
    }
}

rust_i18n::i18n!("locales", backend = ThagI18n::new());
