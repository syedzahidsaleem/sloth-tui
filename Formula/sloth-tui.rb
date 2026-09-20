class SlothTui < Formula
  desc "Terminal interface for movies, anime, sports, F1, and live TV"
  homepage "https://github.com/syedzahidsaleem/sloth-tui"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on "mpv"

  on_macos do
    on_intel do
      url "https://github.com/syedzahidsaleem/sloth-tui/releases/download/v0.1.0/sloth-tui-macos-x86_64.tar.gz"
      sha256 "fd264700a4a10baed4c1d382c8417764a3ae244218ce89742d30e544f213e26e"
    end
    on_arm do
      url "https://github.com/syedzahidsaleem/sloth-tui/releases/download/v0.1.0/sloth-tui-macos-aarch64.tar.gz"
      sha256 "49ec93ab2e29a0140bbe5205c8e716973eac2962f17307622fd90cecb72466ff"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/syedzahidsaleem/sloth-tui/releases/download/v0.1.0/sloth-tui-linux-x86_64.tar.gz"
      sha256 "ccf04417b7e81ac29edf89c10df155b15ab87c048b62660cad6dbe69a95d9182"
    end
    on_arm do
      url "https://github.com/syedzahidsaleem/sloth-tui/releases/download/v0.1.0/sloth-tui-linux-aarch64.tar.gz"
      sha256 "e931aeb527956e78061bd23f608d43e5f6cccaeb9fa273c7abab04a73d48043d"
    end
  end

  def install
    bin.install "sloth-tui"
  end

  test do
    assert_match "sloth-tui #{version}", shell_output("#{bin}/sloth-tui --version")
  end
end
