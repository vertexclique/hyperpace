URL: https://github.com/torvalds/linux/blob/master/drivers/hid/hidraw.c
Retrieved: 2026-09-15
Source type: Linux kernel source code (mainline master at retrieval time).
Retrieval method: `gh api repos/torvalds/linux/contents/drivers/hid/hidraw.c` (base64 decoded, verbatim).
Character note: none needed.

# Excerpts

`hidraw_send_report` (write path):
```c
static ssize_t hidraw_send_report(struct file *file, const char __user *buffer, size_t count, unsigned char report_type)
{
	...
	if (!hidraw_table[minor] || !hidraw_table[minor]->exist) {
		ret = -ENODEV;
		goto out;
	}

	dev = hidraw_table[minor]->hid;

	if (count > HID_MAX_BUFFER_SIZE) {
		hid_warn(dev, "pid %d passed too large report\n",
			 task_pid_nr(current));
		ret = -EINVAL;
		goto out;
	}

	if (count < 2) {
		hid_warn(dev, "pid %d passed too short report\n",
			 task_pid_nr(current));
		ret = -EINVAL;
		goto out;
	}

	buf = memdup_user(buffer, count);
	...
	if ((report_type == HID_OUTPUT_REPORT) &&
	    !(dev->quirks & HID_QUIRK_NO_OUTPUT_REPORTS_ON_INTR_EP)) {
		ret = __hid_hw_output_report(dev, buf, count, (u64)(long)file, false);
		/*
		 * compatibility with old implementation of USB-HID and I2C-HID:
		 * if the device does not support receiving output reports,
		 * on an interrupt endpoint, fallback to SET_REPORT HID command.
		 */
		if (ret != -ENOSYS)
			goto out_free;
```

`hidraw_read` (grep of the function body):
```c
		return -ENODEV;
...
				if (!list->hidraw->exist) {
					ret = -EIO;
...
					ret = -EAGAIN;
```

`hidraw_poll`:
```c
static __poll_t hidraw_poll(struct file *file, poll_table *wait)
{
	struct hidraw_list *list = file->private_data;
	__poll_t mask = EPOLLOUT | EPOLLWRNORM; /* hidraw is always writable */

	poll_wait(file, &list->hidraw->wait, wait);
	if (list->head != list->tail)
		mask |= EPOLLIN | EPOLLRDNORM;
	if (!list->hidraw->exist || hidraw_is_revoked(list))
		mask |= EPOLLERR | EPOLLHUP;
	return mask;
}
```
