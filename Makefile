build-x86:
	@rm -f *_x86.dmg
	@cargo bundle --target x86_64-apple-darwin --release
	@appdmg assets/spec_x86.json LabelPrep_arm64.dmg
build-arm64:
	@rm -f *_arm64.dmg
	@cargo bundle --release
	@alias bless=/usr/sbin/bless
	@appdmg assets/spec_arm64.json LabelPrep_arm64.dmg

build-all: build-x86 build-arm64

clean:
	@rm -f *.dmg
