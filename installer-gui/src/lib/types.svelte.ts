export class ArgMenuInputData {
    booleans: Record<string, boolean> = $state({});
    strings: Record<string, string> = $state({});
}

export interface InstallerCommand {
    subcommands: InstallerSubcommand[];
}

export interface InstallerSubcommand {
    arguments: InstallerArgument[];
    command: string;
    label: string;
    show_device_network_setup: boolean;
}

export interface InstallerArgument {
    advanced: boolean;
    flag: string;
    help: string;
    label: string;
    takes_values: boolean;
}
