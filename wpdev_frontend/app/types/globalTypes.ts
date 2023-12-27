type uuid = string;

export interface Instance {
    uuid: uuid;
    status: InstanceStatus;
    containers: Container[];
    nginx_port: number;
    adminer_port: number;
    wordpress_data: WordPressData;
}

export interface WordPressData {
    site_url: string;
    adminer_url: string;
}

export interface Container {
    container_id: uuid;
    container_image: ContainerImage;
    container_status: ContainerStatus;
}

export enum ContainerImage {
    Adminer = "adminer",
    Wordpress = "wordpress",
    Nginx = "nginx",
    MySQL = "mysql",
    Unknown = "unknown",
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
