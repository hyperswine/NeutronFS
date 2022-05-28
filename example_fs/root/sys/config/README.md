# Kernel Config

What actually happens when you specify your config here is that the kernel serialiser serialises and encrypts that data into /sys/.kernel_config. On boot, the kernel loads and decodes that data to load modules and settings.

If a file is not properly configured, the kernel checker kerncheck.spx should give you a warning. That is if your using a supported program like nano or vscode.
