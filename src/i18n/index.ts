import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en/common.json";
import zh from "./zh-CN/common.json";
import enConn from "./en/connections.json";
import zhConn from "./zh-CN/connections.json";

void i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { common: zh, connections: zhConn },
    en: { common: en, connections: enConn },
  },
  lng: "zh-CN",
  fallbackLng: "en",
  defaultNS: "common",
  interpolation: { escapeValue: false },
});

export default i18n;
