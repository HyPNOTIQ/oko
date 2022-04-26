use ash::vk;

pub struct PhysicalDevice {
	handle: vk::PhysicalDevice,
	queue_families_properties: Vec<vk::QueueFamilyProperties>,
	properties: vk::PhysicalDeviceProperties,
	features: vk::PhysicalDeviceFeatures,
}

impl PhysicalDevice {
	pub fn new(
		handle: vk::PhysicalDevice,
		queue_families_properties: Vec<vk::QueueFamilyProperties>,
		properties: vk::PhysicalDeviceProperties,
		features: vk::PhysicalDeviceFeatures,
	) -> Self {
		Self {
			handle,
			queue_families_properties,
			properties,
			features,
		}
	}

	pub fn handle(&self) -> vk::PhysicalDevice {
		self.handle
	}

	pub fn queue_families_properties(&self) -> &Vec<vk::QueueFamilyProperties> {
		&self.queue_families_properties
	}
}
