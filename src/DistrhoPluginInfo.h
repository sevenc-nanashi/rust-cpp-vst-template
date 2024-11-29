#ifndef DISTRHO_PLUGIN_INFO_H_INCLUDED
#define DISTRHO_PLUGIN_INFO_H_INCLUDED

#define DISTRHO_PLUGIN_BRAND "Nanashi."
#ifdef DEBUG
#define DISTRHO_PLUGIN_NAME "My Plugin (Debug)"
#else
#define DISTRHO_PLUGIN_NAME "My Plugin"
#endif
#define DISTRHO_PLUGIN_URI "https://github.com/sevenc-nanashi/rust-cpp-webview-vst-template/"

// #define DISTRHO_PLUGIN_BRAND_ID Vcvx
#define DISTRHO_PLUGIN_BRAND_ID ScNs

#ifdef DEBUG
#define DISTRHO_PLUGIN_UNIQUE_ID RCWV
#else
#define DISTRHO_PLUGIN_UNIQUE_ID RCWV
#endif

// #define DISTRHO_PLUGIN_CLAP_ID "jp.hiroshiba.vvvst"
#ifdef DEBUG
#define DISTRHO_PLUGIN_CLAP_ID "com.sevenc-nanashi.rust-cpp-webview-vst-template-debug"
#else
#define DISTRHO_PLUGIN_CLAP_ID "com.sevenc-nanashi.rust-cpp-webview-vst-template"
#endif

#define DISTRHO_PLUGIN_HAS_UI 1
#define DISTRHO_PLUGIN_IS_SYNTH 0
#define DISTRHO_PLUGIN_IS_RT_SAFE 1
#define DISTRHO_PLUGIN_HAS_EMBED_UI 1
#define DISTRHO_PLUGIN_HAS_EXTERNAL_UI 1
#define DISTRHO_PLUGIN_NUM_INPUTS 2
#define DISTRHO_PLUGIN_NUM_OUTPUTS 2
#define DISTRHO_PLUGIN_WANT_TIMEPOS 1
#define DISTRHO_PLUGIN_WANT_STATE 1
#define DISTRHO_PLUGIN_WANT_FULL_STATE 1
#define DISTRHO_PLUGIN_WANT_DIRECT_ACCESS 1
#define DISTRHO_UI_USER_RESIZABLE 1
#define DISTRHO_UI_DEFAULT_WIDTH 1080
#define DISTRHO_UI_DEFAULT_HEIGHT 720

#define DISTRHO_PLUGIN_VST3_CATEGORIES "Instrument|Synth"

#endif // DISTRHO_PLUGIN_INFO_H_INCLUDED
