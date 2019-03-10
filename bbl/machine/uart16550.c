// See LICENSE for license details.

#include <string.h>
#include "uart16550.h"
#include "fdt.h"

volatile uint8_t* uart16550;

#define UART_REG_QUEUE     0
#define UART_REG_LINESTAT  5
#define UART_REG_STATUS_RX 0x01
#define UART_REG_STATUS_TX 0x20

void uart16550_putchar(uint8_t ch)
{
  while ((uart16550[UART_REG_LINESTAT] & UART_REG_STATUS_TX) == 0);
  uart16550[UART_REG_QUEUE] = ch;
}

int uart16550_getchar()
{
  if (uart16550[UART_REG_LINESTAT] & UART_REG_STATUS_RX)
    return uart16550[UART_REG_QUEUE];
  return -1;
}

struct uart16550_scan
{
  int compat;
  uint64_t reg;
};

// 将对应的uart16550_scan置零
static void uart16550_open(const struct fdt_scan_node *node, void *extra)
{
  struct uart16550_scan *scan = (struct uart16550_scan *)extra;
  memset(scan, 0, sizeof(*scan));
}

// 匹配节点中的属性值, 这里只关注两种属性，compatible与reg
static void uart16550_prop(const struct fdt_scan_prop *prop, void *extra)
{
  struct uart16550_scan *scan = (struct uart16550_scan *)extra;
  // 如果是compatible属性,并且属性值和uart16550匹配，那么设置scan->compat为1
  if (!strcmp(prop->name, "compatible") && !strcmp((const char*)prop->value, "ns16550a")) {
    scan->compat = 1;
  } else if (!strcmp(prop->name, "reg")) {  // 如果是region属性的话,记录到scan->reg中
    fdt_get_address(prop->node->parent, prop->value, &scan->reg);
  }
}

// 找到uart16550对应的device tree节点，设置对应的内存信息
static void uart16550_done(const struct fdt_scan_node *node, void *extra)
{
  struct uart16550_scan *scan = (struct uart16550_scan *)extra;
  if (!scan->compat || !scan->reg || uart16550) return;

  uart16550 = (void*)(uintptr_t)scan->reg;  // 这里取的是reg中存储的32位起始地址
  // http://wiki.osdev.org/Serial_Ports
  uart16550[1] = 0x00;    // Disable all interrupts
  uart16550[3] = 0x80;    // Enable DLAB (set baud rate divisor)
  uart16550[0] = 0x03;    // Set divisor to 3 (lo byte) 38400 baud
  uart16550[1] = 0x00;    //                  (hi byte)
  uart16550[3] = 0x03;    // 8 bits, no parity, one stop bit
  uart16550[2] = 0xC7;    // Enable FIFO, clear them, with 14-byte threshold
  uart16550[4] = 0x0B;
  uart16550[1] = 0x01;    // Enable interrupt
}

// 查询设备树中uart16550对应的信息
void query_uart16550(uintptr_t fdt)
{
  struct fdt_cb cb;
  struct uart16550_scan scan;

  memset(&cb, 0, sizeof(cb));
  cb.open = uart16550_open;
  cb.prop = uart16550_prop;
  cb.done = uart16550_done;
  cb.extra = &scan;

  fdt_scan(fdt, &cb);
}
