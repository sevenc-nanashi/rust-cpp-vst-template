#include <choc/platform/choc_DynamicLibrary.h>
#include <choc/platform/choc_Platform.h>
#include <optional>
#include <shared_mutex>
#include <whereami++.hpp>

namespace Rust {
std::optional<choc::file::DynamicLibrary> lib;
std::shared_mutex libMutex;

choc::file::DynamicLibrary *loadRustDll() {
  std::shared_lock lock(libMutex);
  if (lib.has_value()) {
    return &lib.value();
  }
  lock.unlock();

  std::unique_lock ulock(libMutex);
  auto libPath = whereami::module_dir() +
#if defined CHOC_WINDOWS
                 "/my_plugin_impl.dll";
#elif defined CHOC_OSX
                 "/libmy_plugin_impl.dylib";
#else
                 "/libmy_plugin_impl.so";
#endif
  auto localLib = choc::file::DynamicLibrary(libPath);
  lib = std::move(localLib);
  return &lib.value();
}
} // namespace Rust
