CREATE TABLE `tiles` (
  `id` INT UNSIGNED NOT NULL,
  `latitude` INT NOT NULL,
  `longitude` INT NOT NULL,
  `elevation_data` MEDIUMBLOB NOT NULL,
  `imagery_data` MEDIUMBLOB NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=ascii