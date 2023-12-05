type uuid = string;

export interface Instance {
    container_ids: string[];
    uuid: uuid;
    status: InstanceStatus;
    container_statuses: { [key: uuid]: string };
    nginx_port: number;
    adminer_port: number;
    wordpress_data: WordPressData;
}

export interface WordPressData {
    site_url: string;
    adminer_url: string;
}

export enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    Dead,
    Unknown,
    NotFound,
    Deleted,
}

export enum InstanceStatus {
    Running = "Running",
    Stopped = "Stopped",
    Restarting = "Restarting",
    Paused = "Paused",
    Exited = "Exited",
    Dead = "Dead",
    Unknown = "Unknown",
    PartiallyRunning = "PartiallyRunning",
}
