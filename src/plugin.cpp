#include "plugin.hpp"
#include "DistrhoDetails.hpp"
#include "DistrhoPlugin.hpp"
#include "rust.generated.hpp"
#include <format>
#include <string>
// -----------------------------------------------------------------------------------------------------------

MyPlugin::MyPlugin() : Plugin(0, 0, 1) {
  inner = std::shared_ptr<Rust::Plugin>(
      Rust::plugin_new(), [](Rust::Plugin *p) { Rust::plugin_drop(p); });
}

/**
   Get the plugin label.
   A plugin label follows the same rules as Parameter::symbol, with the
   exception that it can start with numbers.
 */
const char *MyPlugin::getLabel() const {
#ifdef DEBUG
  return "my_plugin_debug";
#else
  return "my_plugin";
#endif
}

/**
   Get an extensive comment/description about the plugin.
 */
const char *MyPlugin::getDescription() const {
  return "Example VST plugin written in Rust and C++";
}

/**
   Get the plugin author/maker.
 */
const char *MyPlugin::getMaker() const {
  return "Nanashi. (https://sevenc7c.com)";
}

/**
   Get the plugin homepage.
 */
const char *MyPlugin::getHomePage() const {
  return "https://github.com/sevenc-nanashi/rust-cpp-vst-template/";
}

/**
   Get the plugin license name (a single line of text).
   For commercial plugins this should return some short copyright information.
 */
const char *MyPlugin::getLicense() const { return "LGPLv3"; }

/**
   Get the plugin version, in hexadecimal.
 */
uint32_t MyPlugin::getVersion() const {
  auto version = Rust::get_version();

  return d_version(version.major, version.minor, version.patch);
}

/* --------------------------------------------------------------------------------------------------------
 * Init */

/**
   Initialize the audio port @a index.@n
   This function will be called once, shortly after the plugin is created.
 */
void MyPlugin::initAudioPort(bool input, uint32_t index, AudioPort &port) {
  port.groupId = index / 2;
  auto name = std::format("Channel {}", index / 2 + 1);
  port.name = String(name.c_str());

  auto symbol = std::format("channel_{}", index / 2 + 1);
  port.symbol = String(symbol.c_str());
}

void MyPlugin::initState(uint32_t index, State &state) {
  state.defaultValue = "";
  state.key = "state";
  state.hints = kStateIsBase64Blob;
}
void MyPlugin::setState(const char *key, const char *value) {
  Rust::plugin_set_state(inner.get(), value);
}
String MyPlugin::getState(const char *key) const {
  auto stateStringPtr = Rust::plugin_get_state(inner.get());
  auto stateStdString = std::string(stateStringPtr);
  Rust::cstring_drop(stateStringPtr);

  return String(stateStdString.c_str());
}

/* --------------------------------------------------------------------------------------------------------
 * Process */

/**
   Run/process function for plugins without MIDI input.
 */
void MyPlugin::run(const float **inputs, float **outputs, uint32_t frames) {
  auto sampleRate = this->getSampleRate();
  auto timePosition = this->getTimePosition();
  // timePosition.frame is uint64_t, but in Cubase timePosition.frame sometimes
  // becomes negative so we need to cast it to int64_t
  int64_t samplePosition = timePosition.frame;
  auto isPlaying = timePosition.playing;
  Rust::plugin_run(inner.get(), inputs, outputs, sampleRate, frames, isPlaying,
                   samplePosition);
}

START_NAMESPACE_DISTRHO
Plugin *createPlugin() { return new MyPlugin(); }
END_NAMESPACE_DISTRHO
