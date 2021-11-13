
/// URL to download the template from
pub const TEMPLATE_FILE: &str =
    "https://github.com/espressif/esp-idf-template/archive/refs/heads/master.zip";

/// IDF C template
pub const C_TEMPLATE: &str = r#"#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"


void app_main(void)
{
    // TODO Insert code
}
"#;

/// IDF C++ template, it requires extern "C" due to link requirements
pub const CPP_TEMPLATE: &str = r#"#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"


extern "C" void app_main(void)
{
    // TODO Insert code
}
"#;