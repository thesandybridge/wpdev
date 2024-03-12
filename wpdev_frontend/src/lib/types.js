/**
 * @typedef {string} uuid A string that represents a universally unique identifier.
 */

/**
 * Represents the data of a WordPress site.
 * @typedef {Object} WordPressData
 * @property {string} site_url The URL of the WordPress site.
 * @property {string} adminer_url The URL to access Adminer for database management.
 */

/**
 * Represents the status of a container.
 * @enum {string}
 */
const ContainerStatus = {
    Running: "Running",
    Stopped: "Stopped",
    Restarting: "Restarting",
    Paused: "Paused",
    Exited: "Exited",
    Dead: "Dead",
    Unknown: "Unknown",
    NotFound: "NotFound",
    Deleted: "Deleted",
};

/**
 * Represents the image of a container.
 * @enum {string}
 */
const ContainerImage = {
    Adminer: "adminer",
    Wordpress: "wordpress",
    Nginx: "nginx",
    MySQL: "mysql",
    Unknown: "unknown",
};

/**
 * Represents a container within an instance.
 * @typedef {Object} Container
 * @property {uuid} container_id The ID of the container.
 * @property {ContainerImage} container_image The image of the container.
 * @property {ContainerStatus} container_status The status of the container.
 */

/**
 * Represents the status of an instance.
 * @enum {string}
 */
const InstanceStatus = {
    Running: "Running",
    Stopped: "Stopped",
    Restarting: "Restarting",
    Paused: "Paused",
    Exited: "Exited",
    Dead: "Dead",
    Unknown: "Unknown",
    PartiallyRunning: "PartiallyRunning",
};

/**
 * Represents an instance containing multiple containers and configuration for WordPress.
 * @typedef {Object} Instance
 * @property {uuid} uuid The universally unique identifier of the instance.
 * @property {InstanceStatus} status The status of the instance.
 * @property {Container[]} containers An array of containers within the instance.
 * @property {number} nginx_port The port number used by Nginx.
 * @property {number} adminer_port The port number used by Adminer.
 * @property {WordPressData} wordpress_data Data specific to the WordPress installation.
 */

