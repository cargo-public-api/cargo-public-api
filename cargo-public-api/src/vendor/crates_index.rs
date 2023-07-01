//! This module is vendoring of code from
//! <https://github.com/frewsxcv/rust-crates-index> with minor modification to
//! avoid the heavy git2 dependency until crates-index 0.20.0 with
//! <https://github.com/frewsxcv/rust-crates-index/pull/107> has been released.

// Copyright 2015 Corey Farwell
// Copyright 2015 Contributors of github.com/huonw/crates.io-graph
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// A copy of the License as per 4 (a) of the License can be found at the bottom
// of this file.

#![allow(unused)]

use rustc_hash::FxHashSet;
use serde::Deserialize;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::sync::Arc;

/// A single version of a crate (package) published to the index
#[derive(Deserialize, Clone, Debug)]
pub struct Version {
    name: SmolStr,
    vers: SmolStr,
    deps: Arc<[Dependency]>,
    features: Arc<HashMap<String, Vec<String>>>,
    /// It's wrapped in `Option<Box>` to reduce size of the struct when the field is unused (i.e. almost always)
    /// <https://rust-lang.github.io/rfcs/3143-cargo-weak-namespaced-features.html#index-changes>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[allow(clippy::box_collection)]
    features2: Option<Box<HashMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<Box<SmolStr>>,
    #[serde(default)]
    rust_version: Option<SmolStr>,
    #[serde(with = "hex")]
    cksum: [u8; 32],
    #[serde(default)]
    yanked: bool,
}

impl Version {
    /// Name of the crate
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Name of this version
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.vers
    }

    /// Dependencies for this version
    #[inline]
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.deps
    }

    /// Checksum of the package for this version
    ///
    /// SHA256 of the .crate file
    #[inline]
    #[must_use]
    pub fn checksum(&self) -> &[u8; 32] {
        &self.cksum
    }

    /// Explicit features this crate has. This list is not exhaustive,
    /// because any optional dependency becomes a feature automatically.
    ///
    /// `default` is a special feature name for implicitly enabled features.
    #[inline]
    #[must_use]
    pub fn features(&self) -> &HashMap<String, Vec<String>> {
        &self.features
    }

    /// Exclusivity flag. If this is a sys crate, it informs it
    /// conflicts with any other crate with the same links string.
    ///
    /// It does not involve linker or libraries in any way.
    #[inline]
    #[must_use]
    pub fn links(&self) -> Option<&str> {
        self.links.as_ref().map(|s| s.as_str())
    }

    /// Whether this version was [yanked](http://doc.crates.io/crates-io.html#cargo-yank) from the
    /// index
    #[inline]
    #[must_use]
    pub fn is_yanked(&self) -> bool {
        self.yanked
    }

    /// Required version of rust
    ///
    /// Corresponds to `package.rust-version`.
    ///
    /// Added in 2023 (see <https://github.com/rust-lang/crates.io/pull/6267>),
    /// can be `None` if published before then or if not set in the manifest.
    #[inline]
    #[must_use]
    pub fn rust_version(&self) -> Option<&str> {
        self.rust_version.as_deref()
    }
}

/// A single dependency of a specific crate version
#[derive(Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Dependency {
    name: String,
    req: String,
    /// Double indirection to remove size from this struct, since the features are rarely set
    features: Box<Box<[String]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<DependencyKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    optional: bool,
    default_features: bool,
}

impl Dependency {
    /// Dependency's arbitrary nickname (it may be an alias). Use [`Dependency::crate_name`] for actual crate name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Semver version pattern
    #[inline]
    #[must_use]
    pub fn requirement(&self) -> &str {
        &self.req
    }

    /// Features unconditionally enabled when using this dependency,
    /// in addition to [`Dependency::has_default_features`] and features enabled through
    /// parent crate's feature list.
    #[inline]
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }

    /// If it's optional, it implies a feature of its [`Dependency::name`], and can be enabled through
    /// the crate's features.
    #[inline]
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    /// If `true` (default), enable `default` feature of this dependency
    #[inline]
    #[must_use]
    pub fn has_default_features(&self) -> bool {
        self.default_features
    }

    /// Returns the name of the crate providing the dependency.
    /// This is equivalent to `name()` unless `self.package()`
    /// is not `None`, in which case it's equal to `self.package()`.
    ///
    /// Basically, you can define a dependency in your `Cargo.toml`
    /// like this:
    ///
    /// ```toml
    /// serde_lib = {version = "1", package = "serde"}
    /// ```
    ///
    /// ...which means that it uses the crate `serde` but imports
    /// it under the name `serde_lib`.
    #[inline]
    #[must_use]
    pub fn crate_name(&self) -> &str {
        match self.package {
            Some(ref s) => s,
            None => self.name(),
        }
    }
}

/// Section in which this dependency was defined
#[derive(Debug, Copy, Clone, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    /// Used at run time
    Normal,
    /// Not fetched and not used, except for when used directly in a workspace
    Dev,
    /// Used at build time, not available at run time
    Build,
}

/// A whole crate with all its versions
#[derive(Deserialize, Clone, Debug)]
pub struct Crate {
    versions: Box<[Version]>,
}

impl Crate {
    /// All versions of this crate sorted chronologically by date originally published
    ///
    /// Warning: may be yanked or duplicate
    #[inline]
    #[must_use]
    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    /// The highest version as per semantic versioning specification
    ///
    /// Warning: may be pre-release or yanked
    #[must_use]
    pub fn highest_version(&self) -> &Version {
        self.versions
            .iter()
            .max_by_key(|v| semver::Version::parse(&v.vers).ok())
            // Safety: Versions inside the index will always adhere to
            // semantic versioning. If a crate is inside the index, at
            // least one version is available.
            .unwrap()
    }

    /// Crate's unique registry name. Case-sensitive, mostly.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.versions[0].name()
    }

    /// Parse crate file from in-memory JSON-lines data
    #[inline]
    pub fn from_slice(bytes: &[u8]) -> std::io::Result<Crate> {
        let mut dedupe = DedupeContext::new();
        Self::from_slice_with_context(bytes, &mut dedupe)
    }

    /// Parse crate file from in-memory JSON data
    #[inline(never)]
    pub(crate) fn from_slice_with_context(
        mut bytes: &[u8],
        dedupe: &mut DedupeContext,
    ) -> io::Result<Crate> {
        fn is_newline(&c: &u8) -> bool {
            c == b'\n'
        }

        // Trim last newline
        while bytes.last() == Some(&b'\n') {
            bytes = &bytes[..bytes.len() - 1];
        }

        let num_versions = bytes.split(is_newline).count();
        let mut versions = Vec::with_capacity(num_versions);
        for line in bytes.split(is_newline) {
            let mut version: Version = serde_json::from_slice(line)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            if let Some(features2) = version.features2.take() {
                if let Some(f1) = Arc::get_mut(&mut version.features) {
                    for (key, mut val) in features2.into_iter() {
                        f1.entry(key).or_insert_with(Vec::new).append(&mut val);
                    }
                }
            }

            // Many versions have identical dependencies and features
            dedupe.deps(&mut version.deps);
            dedupe.features(&mut version.features);

            versions.push(version);
        }
        if versions.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        debug_assert_eq!(versions.len(), versions.capacity());
        Ok(Crate {
            versions: versions.into_boxed_slice(),
        })
    }
}

/// Many crates (their versions) have the same features and dependencies
pub(crate) struct DedupeContext {
    features: FxHashSet<HashableHashMap<String, Vec<String>>>,
    deps: FxHashSet<Arc<[Dependency]>>,
}

impl DedupeContext {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            deps: FxHashSet::default(),
            features: FxHashSet::default(),
        }
    }

    pub(crate) fn features(&mut self, features: &mut Arc<HashMap<String, Vec<String>>>) {
        let features_to_dedupe = HashableHashMap::new(Arc::clone(features));
        if let Some(has_feats) = self.features.get(&features_to_dedupe) {
            *features = Arc::clone(&has_feats.map);
        } else {
            if self.features.len() > 16384 {
                // keeps peak memory low (must clear, remove is leaving tombstones)
                self.features.clear();
            }
            self.features.insert(features_to_dedupe);
        }
    }

    pub(crate) fn deps(&mut self, deps: &mut Arc<[Dependency]>) {
        if let Some(has_deps) = self.deps.get(&*deps) {
            *deps = Arc::clone(has_deps);
        } else {
            if self.deps.len() > 16384 {
                // keeps peak memory low (must clear, remove is leaving tombstones)
                self.deps.clear();
            }
            self.deps.insert(Arc::clone(deps));
        }
    }
}

/// New type that caches hash of the hashmap (the default hashmap has a random order of the keys, so it's not cheap to hash)
#[derive(PartialEq, Eq)]
pub struct HashableHashMap<K: PartialEq + Hash + Eq, V: PartialEq + Hash + Eq> {
    pub map: Arc<HashMap<K, V>>,
    hash: u64,
}

impl<K: PartialEq + Hash + Eq, V: PartialEq + Hash + Eq> Hash for HashableHashMap<K, V> {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        hasher.write_u64(self.hash);
    }
}

impl<K: PartialEq + Hash + Eq, V: PartialEq + Hash + Eq> HashableHashMap<K, V> {
    pub(crate) fn new(map: Arc<HashMap<K, V>>) -> Self {
        let mut hash = 0;
        for (k, v) in map.iter() {
            let mut hasher = rustc_hash::FxHasher::default();
            k.hash(&mut hasher);
            v.hash(&mut hasher);
            hash ^= hasher.finish(); // XOR makes it order-independent
        }
        Self { map, hash }
    }
}

/*

                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

APPENDIX: How to apply the Apache License to your work.

   To apply the Apache License to your work, attach the following
   boilerplate notice, with the fields enclosed by brackets "[]"
   replaced with your own identifying information. (Don't include
   the brackets!)  The text should be enclosed in the appropriate
   comment syntax for the file format. We also recommend that a
   file or class name and description of purpose be included on the
   same "printed page" as the copyright notice for easier
   identification within third-party archives.

Copyright [yyyy] [name of copyright owner]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

*/
