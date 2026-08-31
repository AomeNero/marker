cask "marker" do
  arch arm: "aarch64", intel: "x64"

  version "2.2.0"
  sha256 arm:   "316afc1076e43560dfb67df1ec55892e3c35b35ac08715c3e130930680b0a03c",
         intel: "d770b49125118af5316ddcfe6b12cee00b1e5129ab53b7f66aa88494a0a2adf"

  url "https://github.com/AomeNero/marker/releases/download/v#{version}/Marker_#{version}_#{arch}.dmg",
      verified: "github.com/AomeNero/marker/"
  name "Marker"
  desc "Lightweight screen annotation tool with whiteboard mode"
  homepage "https://marker.cn/"

  app "Marker.app"

  zap trash: [
    "~/Library/Application Support/com.marker.app",
    "~/Library/Preferences/com.marker.app.plist",
    "~/Library/Saved Application State/com.marker.app.savedState",
  ]
end
