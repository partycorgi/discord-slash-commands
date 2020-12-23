.PHONY: build

CRATES = $(dir $(wildcard ./crates/*/))

help: ## Show this help.
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {sub("\\\\n",sprintf("\n%22c"," "), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## build on ci
	apt install musl-dev musl-tools
	@echo $(CRATES)
	for crate in $(CRATES) ; do \
		cargo install --path $$crate --root bins --target x86_64-unknown-linux-musl ;\
	done
	@mkdir -p functions
	@cp bins/bin/* functions/

create-guild-command:  ## Create a guild-scoped command. Guild commands update instantly and should be used for testing
	@curl -XPOST \
	  -H "Authorization: Bot $DISCORD_BOT_TOKEN" \
	  -d @./crates/repl/options.json \
	  https://discord.com/api/v8/applications/714618235458289804/guilds/601625579166367755/commands 
	# For authorization, you can use either your bot token 
	# headers = {
	# 	"Authorization": "Bot 123456"
	# }

	# # or a client credentials token for your app with the applications.commmands.update scope
	# headers = {
	# 	"Authorization": "Bearer abcdefg"
	# }
create-global-command:  ## Create a global command, global commands are cached for 1h and should be used for "making public" the command
	@echo "not implemented yet"