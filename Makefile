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
	sudo setcap "cap_dac_override+ep" $(OUT_DIR)/$(BINARY_NAME)

clean:
	$(RM) -r $(OUT_DIR)
	docker rm extract || true
	docker rmi $(IMAGE_NAME) || true

.PHONY: build clean