from conans import ConanFile, tools
from conan.tools.cmake import CMakeToolchain, CMake
import re
import os
import shutil
import glob


def copy_files_with_extension(src_dir, dst_dir, extension):
    files = glob.glob(os.path.join(src_dir, f"*.{extension}"))
    for file in files:
        shutil.copy2(file, dst_dir)


class RerunConan(ConanFile):
    name = "rerun-sdk-cpp"
    license = "MIT"
    url = "https://github.com/rerun-io/rerun"
    description = "Rerun - Visualization"
    settings = "os", "build_type", "arch"
    options = {
        "shared": [True, False],
        "download_arrow": [True, False],
        "fPIC": [True, False]
    }
    default_options = {
        "shared": False,
        "download_arrow": True,
        "fPIC": True,
    }

    generators = "CMakeToolchain"

    exports_sources = [
        "rerun_cpp/docs/*",
        "rerun_cpp/src/*",
        "rerun_cpp/arrow_cpp_install.md",
        "rerun_cpp/CMakeLists.txt",
        "rerun_cpp/cmake_setup_in_detail.md",
        "rerun_cpp/Config.cmake.in",
        "rerun_cpp/download_and_build_arrow.cmake",
        "rerun_cpp/README.md",
        "LICENSE-APACHE",
        "LICENSE-MIT",
        "Cargo.toml",
    ]

    def set_version(self):
        # Read Cargo.toml to extract the version
        cargo_toml = tools.load("Cargo.toml")
        version_match = re.search(r'\nversion\s*=\s*"([a-zA-Z0-9\.\+-]+)"', cargo_toml)
        if version_match:
            full_version = version_match.group(1)
            # Extract the base version
            base_version = re.match(r'^(\d+\.\d+\.\d+)', full_version).group(1)
            self.version = base_version
        else:
            raise Exception("Could not find version in Cargo.toml")

    def generate(self):
        tc = CMakeToolchain(self)
        tc.variables["BUILD_SHARED_LIBS"] = str(self.options.shared)
        tc.variables["RERUN_DOWNLOAD_AND_BUILD_ARROW"] = str(self.options.download_arrow)
        tc.generate()

    def source(self):
        # Download the appropriate precompiled binary based on the version
        version = self.version
        tools.download("https://github.com/rerun-io/rerun/releases/download/" + self.version + "/rerun_cpp_sdk.zip",
                       "rerun_cpp_sdk.zip")

        tools.unzip("rerun_cpp_sdk.zip")

    def configure_cmake(self):
        cmake = CMake(self)
        return cmake

    def build(self):
        shutil.copytree(os.path.join("rerun_cpp_sdk", "lib"), os.path.join("rerun_cpp", "lib"))

        # Let CMake handle the entire build process
        cmake = self.configure_cmake()
        cmake.configure(build_script_folder=os.path.join(self.source_folder, "rerun_cpp"))
        cmake.build()

    def package(self):
        copy_files_with_extension("rerun_cpp_sdk/include", "include", "h")
        copy_files_with_extension("lib", "lib", "a")
        copy_files_with_extension(os.path.join("arrow", "lib"), "lib", "a")
        cmake = self.configure_cmake()
        cmake.install()

    def package_info(self):
        self.cpp_info.libs = tools.collect_libs(self)
        self.cpp_info.libs.extend(["arrow"])
        self.cpp_info.includedirs = ['include']
        self.cpp_info.libdirs = ['lib']
        self.cpp_info.set_property("cmake_file_name", "rerun_sdk_cpp")
        self.cpp_info.filenames["cmake_find_package"] = "rerun_sdk_cpp"
        self.cpp_info.filenames["cmake_find_package_multi"] = "rerun_sdk_cpp"
        self.cpp_info.names["cmake_find_package"] = "rerun_sdk_cpp"
        self.cpp_info.names["cmake_find_package_multi"] = "rerun_sdk_cpp"
        if self.settings.os == "Windows":
            self.cpp_info.system_libs.extend(["ws2_32", "Userenv", "ntdll", "Bcrypt"])
