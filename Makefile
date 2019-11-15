.PHONY: start_vault # Run vault in docker instance
start_vault:
	docker run --rm -e VAULT_DEV_ROOT_TOKEN_ID=sample_root_token_id --cap-add=IPC_LOCK -d -p 8200:8200 --name vault vault
	until vault status -address=http://127.0.0.1:8200; do \
	  echo 'waiting for vault...'; \
	  sleep 1; \
	done

.PHONY: stop_vault # Stop running vault instance
stop_vault:
	docker stop $$(docker ps | grep vault | awk '{print $$1}')

.PHONY: configure_vault # Configure local vault instance
configure_vault: export VAULT_ADDR=http://127.0.0.1:8200
configure_vault:
	vault login sample_root_token_id
	vault secrets enable -version=2 kv
	vault kv put secret/many_secrets/secret-one my-value=s3cr3t1
	vault kv put secret/many_secrets/secret-one my-value=s3cr3t1v2
	vault kv put secret/many_secrets/secret-two my-value=s3cr3t2
	vault kv put secret/secret-three my-value=s3cr3t3 my-other-value=other-s3cr3t3

.PHONY: setup_vault # Start and configure vault
setup_vault: start_vault configure_vault

.PHONY: run # build and run the filesystem
run:
	cargo run ~/tmp/vaultfs config.toml
	# cargo run ~/tmp/vaultfs token

.PHONY: cleanup # unmount folder where system was mounted (not done by app yet)
cleanup:
	fusermount -u ~/tmp/vaultfs

.PHONY: help # Generate list of targets with descriptions
help:
	@grep '^.PHONY: .* #' Makefile | sed "s/\.PHONY: \(.*\) # \(.*\)/$$(tput setaf 2)\1|$$(tput sgr0) \2/" | column -t -s'|'
