#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/multicore.h"
#include "hardware/irq.h"
#include "hardware/pwm.h"



typedef enum {
    None = 0,
    Sync = 1,
    Id = 2,
    Length = 3,
    Command = 4,
    Parameters = 5,
    Checksum = 6,
    Done = 7,
} CommandState;

static int core0_rx_value = 0;
static bool core0_rx_ready = false;

void core0_sio_irq() {
    while (multicore_fifo_rvalid()) {
        core0_rx_value = multicore_fifo_pop_blocking();
        core0_rx_ready = true;
    }

    multicore_fifo_clear_irq();
}

void core0_entry() {
    irq_set_exclusive_handler(SIO_FIFO_IRQ_NUM(0), core0_sio_irq);
    irq_set_enabled(SIO_FIFO_IRQ_NUM(0), true);

    sleep_ms(10);

    CommandState command_state = None;
    int servo_id = 0;
    int length = 0;
    int command = 0;
    int parameter_index = 0;
    int parameters[8];
    int parameter_sum = 0;
    int checksum = 0;

    while (true) {
        char next = getchar();

        switch (command_state) {
        case None:
            if (next == 0x55) {
                command_state = Sync;
            }
            break;
        case Sync:
            if (next == 0x55) {
                command_state = Id;
            }
            else {
                command_state = None;
            }
            break;
        case Id:
            servo_id = next;
            command_state = Length;
            break;
        case Length:
            length = next;
            command_state = Command;
            break;
        case Command:
            command = next;

            parameter_index = 0;
            parameter_sum = 0;

            if (length <= 3) {
                command_state = Checksum;
            }
            else {
                command_state = Parameters;
            }
            break;
        case Parameters:
            parameters[parameter_index] = next;
            parameter_sum += next;
            parameter_index++;

            if (parameter_index >= (length - 3)) {
                command_state = Checksum;
            }
            break;
        case Checksum:
            checksum = next;
            int target_checksum = (255 - ((servo_id + length + command + parameter_sum) % 256));

            // TODO: Reenable checksum
            // if (checksum == target_checksum) {
            command_state = Done;
            // }
            // else {
            //     command_state = None;
            // }
            break;
        case Done:
            break;
        }

        if (command_state == Done) {
            switch (command) {
            case 1:;
                int position = parameters[0] + (parameters[1] << 8);

                if (position > 500) {
                    multicore_fifo_push_blocking(1);
                }
                else {
                    multicore_fifo_push_blocking(0);
                }

                break;
            default:
                break;
            }

            command_state = None;
        }
    }
}



static int core1_rx_value = 0;
static bool core1_rx_ready = false;

void core1_sio_irq() {
    while (multicore_fifo_rvalid()) {
        core1_rx_value = multicore_fifo_pop_blocking();
        core1_rx_ready = true;
    }

    multicore_fifo_clear_irq();
}

void core1_entry() {
    multicore_fifo_clear_irq();

    irq_set_exclusive_handler(SIO_FIFO_IRQ_NUM(1), core1_sio_irq);
    irq_set_enabled(SIO_FIFO_IRQ_NUM(1), true);

    sleep_ms(10);

    gpio_init(PICO_DEFAULT_LED_PIN);
    gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);

    gpio_set_function(4, GPIO_FUNC_PWM);
    uint slice_num = pwm_gpio_to_slice_num(4);
    uint channel_num = pwm_gpio_to_channel(4);

    pwm_set_wrap(slice_num, 512);
    pwm_set_chan_level(slice_num, channel_num, 100);

    pwm_set_enabled(slice_num, true);

    while (true) {
        if (core1_rx_ready) {
            printf("core 2 rx\n");
            gpio_put(PICO_DEFAULT_LED_PIN, core1_rx_value);

            core1_rx_ready = false;

            if (core1_rx_value) {
                pwm_set_chan_level(slice_num, channel_num, 100);
                sleep_ms(3000);
                pwm_set_chan_level(slice_num, channel_num, 256);
                sleep_ms(5000);
                pwm_set_chan_level(slice_num, channel_num, 412);
                sleep_ms(3000);
                pwm_set_chan_level(slice_num, channel_num, 256);
            }
        }
        else {
            sleep_ms(10);
        }
    }
}



int main() {
    stdio_init_all();

    multicore_reset_core1();
    sleep_ms(100);
    multicore_launch_core1(core1_entry);

    core0_entry();
}
