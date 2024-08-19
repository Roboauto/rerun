# Upgrading `rerun-sdk-cpp` Package: A Comprehensive Guide

This guide provides detailed instructions to upgrade the `rerun-sdk-cpp` package to a specified tagged release version. It covers setting up remotes, fetching updates, creating branches, applying necessary changes, building the Conan package, and finally uploading it to your Conan repository.

---

## Prerequisites

- **Git**: Ensure that Git is installed on your system. If not, download it from [here](https://git-scm.com/downloads).

- **Conan**: Ensure that Conan is installed. If not, follow the installation guide [here](https://docs.conan.io/en/latest/installation.html).

- **Access to the `rerun-io/rerun` Repository**: Ensure you have the necessary permissions to clone and fetch from this repository.

---

## Step-by-Step Instructions

### Step 1: Ensure Upstream Remote is Added

First, verify if the `upstream` remote (i.e., the original repository) is added to your local Git configuration. This remote is crucial for fetching the latest tagged releases.

1. **Check Existing Remotes**

   Run the following command to list all configured remotes:

   ```bash
   git remote -v
   ```

   Look for an entry named `upstream`. If it exists, you can skip adding it. If not, proceed to the next step.

2. **Add Upstream Remote**

   Add the `upstream` remote pointing to the original repository:

   ```bash
   git remote add upstream https://github.com/rerun-io/rerun.git
   ```

### Step 2: Fetch Tags and Updates from Upstream

1. **Fetch All Tags**

   Retrieve all tags from the upstream repository to ensure you have access to the latest release versions:

   ```bash
   git fetch --tags upstream
   ```

2. **Update Local Repository**

   Fetch the latest changes from the upstream repository:

   ```bash
   git fetch upstream
   ```

### Step 3: Create a New Branch from the Desired Tag

1. **Identify the Desired Tag**

   Determine the tagged release version you want to upgrade to. Replace `{TAGGED_VERSION}` with this version (e.g., `0.17.0`).

2. **Checkout to a tagged version**

   ```bash
   git checkout  {TAGGED_VERSION}
   ```

3. **Create a New Branch**

   Create a new branch based on the desired tag and switch to it:

   ```bash
   git checkout -b roboauto_release_{TAGGED_VERSION}
   ```

### Step 4: Apply Custom Changes from Previous Release

Depending on your project's requirements, you might need to incorporate specific changes from the previous release. There are two primary methods to achieve this:

1. **Option A: Cherry-Pick Specific Commits**

   If there are particular commits from the previous release that you want to include:

    - **Identify Commits**: Use `git log` on the previous release branch to find the commit hashes you need.

    - **Cherry-Pick Commits**: Apply these commits to your current branch:

      ```bash
      git cherry-pick <commit_hash1> <commit_hash2> ...
      ```

      *Example:*

      ```bash
      git cherry-pick a1b2c3d4 e5f6g7h8
      ```

2. **Option B: Reset to Previous State and Commit**

   NOTE!!! Do before Step 3. 

   If you prefer to revert to the state of the previous release and then apply the new changes:

    - **Reset to Previous Release**

      ```bash
      git reset HEAD~1
      ```

      In new branch
      ```bash
      git commit -am "conanfile + manual for roboauto release"
      ```

      *Note:* This will overwrite your current changes. Ensure that this is the desired action.

    - **Apply New Changes**: After resetting, you can merge or rebase the new tagged release or manually apply the necessary changes.

### Step 5: Update `Cargo.toml` File

Before building the package, ensure that the `Cargo.toml` file reflects the correct version:

1. **Open `Cargo.toml`**

   Use a text editor to open the `Cargo.toml` file.

2. **Update Version**

   Locate the `version` field and update it to `{TAGGED_VERSION}`.

   *Example:*

   ```toml
   [package]
   name = "rerun-sdk-cpp"
   version = "0.17.0"
   ```

### Step 6: Build the Conan Package

1. **Navigate to Project Root**

   Ensure you're in the root directory of your project:

   ```bash
   cd ~/rerun
   ```

2. **Build the Package**

   Use Conan to create the package:

   ```bash
   conan create . roboauto-conan/stable --build=missing
   ```

    - The `.` specifies the current directory contains the `conanfile.py`.
    - `roboauto-conan/stable` is the user/channel.
    - The `--build=missing` flag tells Conan to build from source if binaries are missing.

### Step 7: Upload the Package to Conan Repository

1. **Upload Package**

   Replace `{TAGGED_VERSION}` with the actual version number:

   ```bash
   conan upload rerun-sdk-cpp/{TAGGED_VERSION}@roboauto-conan/stable -r roboauto --all
   ```

   *Example:*

   ```bash
   conan upload rerun-sdk-cpp/0.17.0@roboauto-conan/stable -r roboauto --all
   ```

    - `-r roboauto` specifies the remote repository name.
    - `--all` uploads all package binaries, not just the recipe.

2. **Verify Upload**

   Ensure that the package has been successfully uploaded by checking your Conan repository or using the `conan search` command.

---

## Conclusion

By meticulously following this guide, you can seamlessly upgrade the `rerun-sdk-cpp` package to your desired release version, incorporate necessary changes, and make it available in your Conan repository for integration into your projects. Regularly updating and managing your dependencies ensures stability, security, and access to the latest features.