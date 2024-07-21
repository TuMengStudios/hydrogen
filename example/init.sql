CREATE DATABASE `hydrogen`
/*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci */
/*!80016 DEFAULT ENCRYPTION='N' */
;

use hydrogen;

-- hydrogen.task_info definition
CREATE TABLE `task_info` (
    `id` bigint NOT NULL AUTO_INCREMENT COMMENT 'task id',
    `name` varchar(255) NOT NULL DEFAULT '' COMMENT 'task title',
    `parser_config` json DEFAULT NULL COMMENT 'parser config',
    `status` smallint NOT NULL DEFAULT '0' COMMENT 'task status running, stop, created ...',
    `src_config` json DEFAULT NULL,
    `dst_config` json DEFAULT NULL,
    `debug_text` json DEFAULT NULL,
    `heartbeat` bigint NOT NULL DEFAULT '0' COMMENT 'last heartbeat',
    `updated_at` bigint NOT NULL DEFAULT '0' COMMENT 'update time',
    `created_at` bigint NOT NULL DEFAULT '0' COMMENT 'created time',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

-- hydrogen.task_log definition
CREATE TABLE `task_log` (
    `id` bigint NOT NULL AUTO_INCREMENT COMMENT 'task log id',
    `task_id` bigint NOT NULL DEFAULT '0' COMMENT 'task id',
    `log_info` text COMMENT 'task log',
    `status` mediumint NOT NULL DEFAULT '0' COMMENT 'log status',
    `created_at` bigint NOT NULL DEFAULT '0' COMMENT 'created time',
    `updated_at` bigint NOT NULL DEFAULT '0' COMMENT 'update time',
    PRIMARY KEY (`id`),
    KEY `idx_task_id` (`task_id`) COMMENT 'task index '
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;
