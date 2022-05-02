use {
	super::{make_api_version, ExtensionName},
	anyhow::Result,
	ash::vk::{self, api_version_major, api_version_minor, api_version_patch},
};

pub struct Instance {
	physical_devices: Vec<PhysicalDevice>,
	inner: ash::Instance,
	entry: ash::Entry,
}

impl Instance {
	pub fn new(
		extensions: &[ExtensionName],
		required_version: u32,
	) -> Result<Self> {
		let entry = unsafe { ash::Entry::load()? };

		let required_version_major = api_version_major(required_version);
		let required_version_minor = api_version_minor(required_version);
		let required_version_patch = api_version_patch(required_version);

		let version_1_0 = make_api_version(1, 0, 0);

		let version_1_0_x_requested = version_1_0
			== make_api_version(
				required_version_major,
				required_version_minor,
				0,
			);

		// It is not possible to check path part for 1.0.x vulkan, so ignore it
		let required_version = if version_1_0_x_requested {
			version_1_0
		} else {
			required_version
		};

		let vulkan_api_version = match entry.try_enumerate_instance_version()? {
			Some(version) => version,
			None => version_1_0,
		};

		let vulkan_api_version = if vulkan_api_version < required_version {
			let error = anyhow::anyhow!(
				"The required vulkan version {}.{}.{} is not available",
				required_version_major,
				required_version_minor,
				required_version_patch
			);

			Err(error)
		} else {
			Ok(required_version)
		}?;

		let app_info =
			vk::ApplicationInfo::builder().api_version(vulkan_api_version);

		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_extension_names(extensions);

		let inner = unsafe { entry.create_instance(&create_info, None)? };

		log::info!(
			"Instance version {}.{}.{} created",
			api_version_major(vulkan_api_version),
			api_version_minor(vulkan_api_version),
			api_version_patch(vulkan_api_version)
		);

		let physical_devices = Instance::collect_physical_devices(&inner)?;

		let instance = Self {
			physical_devices,
			inner,
			entry,
		};

		Ok(instance)
	}

	pub fn entry(&self) -> &ash::Entry {
		&self.entry
	}

	pub fn inner(&self) -> &ash::Instance {
		&self.inner
	}

	fn collect_physical_devices(
		instance: &ash::Instance,
	) -> Result<Vec<PhysicalDevice>> {
		let physical_devices =
			unsafe { instance.enumerate_physical_devices()? };

		let physical_devices: Vec<_> = physical_devices
			.iter()
			.map(|&handle| -> PhysicalDevice {
				unsafe {
					let properties =
						instance.get_physical_device_properties(handle);
					let queue_families_properties = instance
						.get_physical_device_queue_family_properties(handle);
					let features =
						instance.get_physical_device_features(handle);

					log::info!(
						"Physical device collected; name: {}",
						::std::ffi::CStr::from_ptr(
							properties.device_name.as_ptr()
						)
						.to_str()
						.unwrap_or("unknown")
					);

					PhysicalDevice {
						handle,
						queue_families_properties,
						properties,
						features,
					}
				}
			})
			.collect();

		Ok(physical_devices)
	}

	pub fn physical_devices(&self) -> &Vec<PhysicalDevice> {
		&self.physical_devices
	}
}

impl Drop for Instance {
	fn drop(&mut self) {
		unsafe {
			self.inner.destroy_instance(None);
		}
	}
}

pub struct PhysicalDevice {
	pub handle: vk::PhysicalDevice,
	pub queue_families_properties: Vec<vk::QueueFamilyProperties>,
	pub properties: vk::PhysicalDeviceProperties,
	pub features: vk::PhysicalDeviceFeatures,
}
