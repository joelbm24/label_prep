build-x86:
	@rm -f *_x86.dmg
	@cargo bundle --target x86_64-apple-darwin --release
	@hdiutil create /tmp/tmp.dmg -ov -volname "LabelPrepinstall" -fs HFS+ -srcfolder target/x86_64-apple-darwin/release/bundle/osx/
	@hdiutil convert /tmp/tmp.dmg -format UDZO -o LabelPrepInstall_x86.dmg
build-arm64:
	@rm -f *_arm64.dmg
	@cargo bundle --release
	@hdiutil create /tmp/tmp.dmg -ov -volname "LabelPrepinstall" -fs HFS+ -srcfolder target/release/bundle/osx/
	@hdiutil convert /tmp/tmp.dmg -format UDZO -o LabelPrepInstall_arm64.dmg

build-all: build-x86 build-arm64

clean:
	@rm -f *.dmg
