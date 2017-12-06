class Mathbib < Formula
  desc "A CLI for finding BibTeX references from MathSciNet"
  homepage "https://github.com/chnn/mathbib"
  url "https://github.com/chnn/mathbib/releases/download/v0.1.0/mathbib.zip"
  sha256 "456e71860dd42d5ce4df405de9e9eca7a18b0a1e087f9bb58ea337fdd7a1a723"

  bottle :unneeded

  def install
    bin.install "mathbib"
  end
end
