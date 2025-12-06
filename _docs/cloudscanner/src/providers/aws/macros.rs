// AWS service initialization macro to reduce code duplication

macro_rules! aws_service_init {
    (
        $self:ident, $has_aws_creds:ident, $service_name:literal, $config_field:ident, $discovery_impl:ty, $mock_discovery:ty
    ) => {
        info!("Initializing {}", $service_name);
        if $has_aws_creds {
            if let Err(e) = $self.validate_aws_credentials() {
                warn!("AWS credential validation failed: {}, using mock", e);
                $self.$config_field = Some(Box::new(<$mock_discovery>::default()));
            } else {
                let config = $self.config.clone();
                match <$discovery_impl>::new(config).await {
                    Ok(discovery) => {
                        info!("{} discovery initialized successfully", $service_name);
                        $self.$config_field = Some(Box::new(discovery));
                    }
                    Err(e) => {
                        let sanitized_error = $self.sanitize_error_message(&e.to_string());
                        warn!("Failed to create {} client: {}, using mock", $service_name, sanitized_error);
                        $self.$config_field = Some(Box::new(<$mock_discovery>::default()));
                    }
                }
            }
        } else {
            info!("No AWS credentials found, using mock {} discovery", $service_name);
            $self.$config_field = Some(Box::new(<$mock_discovery>::default()));
        }
    };
}

// AWS service discovery macro to reduce code duplication
macro_rules! aws_service_discover {
    (
        $self:ident, $service_name:literal, $discovery_field:ident, $discover_method:ident, $all_resources:ident
    ) => {
        if let Some(ref $discovery_field) = $self.$discovery_field {
            match $discovery_field.$discover_method().await {
                Ok(resources) => {
                    info!("Discovered {} {}", resources.len(), $service_name);
                    $all_resources.extend(resources);
                }
                Err(e) => {
                    error!("Failed to discover {}: {}", $service_name, e);
                    // Continue with other services even if one fails
                }
            }
        }
    };
}

// ECS service discovery macro (for multiple discovery methods)
macro_rules! aws_ecs_service_discover {
    (
        $self:ident, $ecs_discovery:ident, $service_name:literal, $discover_method:ident, $all_resources:ident
    ) => {
        if let Some(ref $ecs_discovery) = $self.$ecs_discovery {
            match $ecs_discovery.$discover_method().await {
                Ok(resources) => {
                    info!("Discovered {} {}", resources.len(), $service_name);
                    $all_resources.extend(resources);
                }
                Err(e) => {
                    error!("Failed to discover {}: {}", $service_name, e);
                    // Continue with other services even if one fails
                }
            }
        }
    };
}