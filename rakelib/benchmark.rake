require 'tempfile'
require 'fileutils'
require 'securerandom'

namespace :benchmark do
  file 'target/release/deduplicator' do
    sh "cargo build --release"
  end

  task :few_large_files => 'target/release/deduplicator' do
    # Benchmark 1: ./target/release/deduplicator bench_artifacts
    # Time (mean ± σ):       2.5 ms ±   0.6 ms    [User: 2.4 ms, System: 4.8 ms]
    # Range (min … max):     1.8 ms …   9.7 ms    1474 runs

    root = "bench_artifacts"
    FileUtils.rm_rf(root)
    Dir.mkdir(root)

    # files with same size
    puts "generating files of same size ..."
    2.times.map do |i|
      File.open(File.join(root, "file_#{i}_fwss.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * 100_000))
      end
    end

    # files with different sizes
    puts "generating files of different sizes ..."
    2.times.map do |i|
      File.open(File.join(root, "file_#{i}_fwds.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * (rand * 100_000).ceil))
      end
    end

    # files with same content & size
    puts "generating files of same content and sizes ..."
    2.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwscas.bin"), 'wb') do |f|
        f.write("\0" * (4096 * 100_000))
      end
    end

    # files with different content but same size
    puts "generating files of different content but same sizes ..."
    2.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwdcbss.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * 100_000))
      end
    end

    sh("hyperfine -N --warmup 80 './target/release/deduplicator #{root}'")
    sh("dust '#{root}'")

    FileUtils.rm_rf(root)
  end

  task :many_small_files => 'target/release/deduplicator' do
    # Benchmark 1: ./target/release/deduplicator bench_artifacts
    # Time (mean ± σ):      10.6 ms ±   1.0 ms    [User: 20.0 ms, System: 22.5 ms]
    # Range (min … max):     8.4 ms …  14.2 ms    235 runs

    root = "bench_artifacts"
    Dir.mkdir(root)

    # files with same size
    puts "generating 1000 files of the same size ... "
    1000.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwss.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * 1000))
      end
    end

    # files with different sizes
    puts "generating 1000 files of different sizes ... "
    1000.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwds.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * (rand * 100).ceil))
      end
    end

    # files with same content & size
    puts "generating files of same content and sizes ..."
    1000.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwscas.bin"), 'wb') do |f|
        f.write("\0" * (4096 * 1000))
      end
    end

    # files with different content but same size
    puts "generating files of different content but same sizes ..."
    1000.times.each do |i|
      File.open(File.join(root, "file_#{i}_fwdcbss.bin"), 'wb') do |f|
        f.write(SecureRandom.bytes(4096 * 1000))
      end
    end

    sh("hyperfine --warmup 20 './target/release/deduplicator #{root}'")
    sh("dust '#{root}'")

    FileUtils.rm_rf(root)
  end
end
