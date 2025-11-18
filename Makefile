IMAGE_NAME := mymoulette-builder
BINARY_NAME := mymoulette
OUT_DIR := out

default: build

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

build: $(OUT_DIR)
	docker build -t $(IMAGE_NAME) .
	docker create --name extract $(IMAGE_NAME)
	docker cp extract:/$(BINARY_NAME) $(OUT_DIR)/$(BINARY_NAME)
	docker rm extract
	sudo setcap "cap_sys_chroot,cap_dac_override,cap_setpcap,cap_setfcap,cap_sys_admin,cap_net_raw+ep" $(OUT_DIR)/$(BINARY_NAME)

clean:
	$(RM) -r $(OUT_DIR)
	docker rm extract || true
	docker rmi $(IMAGE_NAME) || true

.PHONY: build clean