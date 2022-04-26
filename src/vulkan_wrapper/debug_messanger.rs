use ash::vk::DebugUtilsMessageSeverityFlagsEXT;
use ash::vk::DebugUtilsMessageTypeFlagsEXT;
use {super::Instance, anyhow::Result, ash::vk};

pub struct DebugMessenger<'a> {
	debug_utils_messenger: vk::DebugUtilsMessengerEXT,
	debug_utils: ash::extensions::ext::DebugUtils,
	_instance: &'a Instance,
}

impl<'a> DebugMessenger<'a> {
	pub fn new(
		instance: &'a Instance,
		callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
	) -> Result<Self> {
		if let Ok(instance_layers) = std::env::var("VK_INSTANCE_LAYERS") {
			log::info!("Instance layers: {}", instance_layers);
		} else {
			log::warn!("No validation layers enabled!");
		}

		let debug_utils_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(
				DebugUtilsMessageSeverityFlagsEXT::VERBOSE
					| DebugUtilsMessageSeverityFlagsEXT::INFO
					| DebugUtilsMessageSeverityFlagsEXT::WARNING
					| DebugUtilsMessageSeverityFlagsEXT::ERROR,
			)
			.message_type(
				DebugUtilsMessageTypeFlagsEXT::GENERAL
					| DebugUtilsMessageTypeFlagsEXT::VALIDATION
					| DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
			)
			.pfn_user_callback(callback);

		let debug_utils =
			ash::extensions::ext::DebugUtils::new(instance.entry(), instance.inner());

		let debug_utils_messenger = unsafe {
			debug_utils.create_debug_utils_messenger(&debug_utils_create_info, None)?
		};

		let debug_messanger = Self {
			debug_utils,
			debug_utils_messenger,
			_instance: instance,
		};

		Ok(debug_messanger)
	}
}

impl<'a> Drop for DebugMessenger<'a> {
	fn drop(&mut self) {
		unsafe {
			self.debug_utils
				.destroy_debug_utils_messenger(self.debug_utils_messenger, None);
		}
	}
}
