use std::collections::HashMap;

use super::build::ImageBuildOutputCompression;

/// TODO
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageRegistryOutput {
    pub(crate) name: String,
    pub(crate) push: bool,
    pub(crate) push_by_digest: bool,
    pub(crate) insecure_registry: bool,
    pub(crate) dangling_name_prefix: String,
    pub(crate) name_canonical: bool,
    pub(crate) compression: ImageBuildOutputCompression,
    pub(crate) compression_level: Option<u8>,
    pub(crate) force_compression: bool,
    pub(crate) oci_mediatypes: bool,
    pub(crate) unpack: bool,
    pub(crate) store: bool,
    pub(crate) annotation: HashMap<String, String>,
}

impl ImageRegistryOutput {
    /// TODO
    pub fn builder(name: &str) -> ImageRegistryOutputBuilder {
        ImageRegistryOutputBuilder::new(name)
    }

    fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            ..Default::default()
        }
    }

    pub(crate) fn into_map(self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        attrs.insert(String::from("name"), self.name);

        attrs.insert(String::from("push"), self.push.to_string());

        attrs.insert(
            String::from("push-by-digest"),
            self.push_by_digest.to_string(),
        );

        attrs.insert(
            String::from("registry.insecure"),
            self.insecure_registry.to_string(),
        );

        attrs.insert(
            String::from("dangling_name_prefix"),
            self.dangling_name_prefix,
        );

        attrs.insert(
            String::from("name_canonical"),
            self.name_canonical.to_string(),
        );

        attrs.insert(String::from("compression"), self.compression.to_string());

        if let Some(compression_level) = self.compression_level {
            attrs.insert(
                String::from("compression-level"),
                compression_level.to_string(),
            );
        }

        attrs.insert(
            String::from("force-compression"),
            self.force_compression.to_string(),
        );

        attrs.insert(
            String::from("oci-mediatypes"),
            self.oci_mediatypes.to_string(),
        );

        attrs.insert(String::from("unpack"), self.unpack.to_string());

        attrs.insert(String::from("store"), self.store.to_string());

        for (key, value) in self.annotation {
            attrs.insert(format!("annotation.{}", key), value);
        }

        attrs
    }
}

/// TODO
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageRegistryOutputBuilder {
    inner: ImageRegistryOutput,
}

impl ImageRegistryOutputBuilder {
    /// TODO
    pub fn new(name: &str) -> Self {
        Self {
            inner: ImageRegistryOutput {
                name: String::from(name),
                ..Default::default()
            },
        }
    }

    /// TODO
    pub fn push(mut self, push: bool) -> Self {
        self.inner.push = push;
        self
    }

    /// TODO
    pub fn push_by_digest(mut self, push_by_digest: bool) -> Self {
        self.inner.push_by_digest = push_by_digest;
        self
    }

    /// TODO
    pub fn insecure_registry(mut self, insecure_registry: bool) -> Self {
        self.inner.insecure_registry = insecure_registry;
        self
    }

    /// TODO
    pub fn dangling_name_prefix(mut self, dangling_name_prefix: &str) -> Self {
        self.inner.dangling_name_prefix = String::from(dangling_name_prefix);
        self
    }

    /// TODO
    pub fn name_canonical(mut self, name_canonical: bool) -> Self {
        self.inner.name_canonical = name_canonical;
        self
    }

    /// Compression type, see [buildkit compression
    /// docs](https://docs.docker.com/build/exporters/#compression)
    pub fn compression(mut self, compression: &ImageBuildOutputCompression) -> Self {
        self.inner.compression = compression.to_owned();
        self
    }

    /// Compression level, see [buildkit compression
    /// docs](https://docs.docker.com/build/exporters/#compression)
    pub fn compression_level(mut self, compression_level: u8) -> Self {
        self.inner.compression_level = Some(compression_level);
        self
    }

    /// Forcefully apply compression, see [buildkit compression
    /// docs](https://docs.docker.com/build/exporters/#compression)
    pub fn force_compression(mut self, force_compression: bool) -> Self {
        self.inner.force_compression = force_compression;
        self
    }

    /// Use OCI media types in exporter manifests. Defaults to `true` for `type=oci`, and `false`
    /// for `type=docker`. See [buildkit OCI media types
    /// docs](https://docs.docker.com/build/exporters/#oci-media-types)
    pub fn oci_mediatypes(mut self, oci_mediatypes: bool) -> Self {
        self.inner.oci_mediatypes = oci_mediatypes;
        self
    }

    /// TODO
    pub fn unpack(mut self, unpack: bool) -> Self {
        self.inner.unpack = unpack;
        self
    }

    /// TODO
    pub fn store(mut self, store: bool) -> Self {
        self.inner.store = store;
        self
    }

    /// Attach an annotation with the respective `key` and `value` to the built image, see
    /// [buildkit annotations
    /// docs](https://docs.docker.com/build/exporters/oci-docker/#annotations)
    pub fn annotation(mut self, key: &str, value: &str) -> Self {
        self.inner
            .annotation
            .insert(String::from(key), String::from(value));
        self
    }

    /// TODO
    pub fn consume(self) -> ImageRegistryOutput {
        self.inner
    }
}
