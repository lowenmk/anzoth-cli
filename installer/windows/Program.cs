using System.Diagnostics;
using System.Formats.Tar;
using System.IO.Compression;
using System.Net.Http.Headers;
using System.Security.Cryptography;
using System.Text.Json;

namespace AnzothInstaller;

internal static class Program
{
    private const string DefaultRepository = "lowenmk/anzoth-cli";
    private const string DefaultInstallRoot = @"%LOCALAPPDATA%\Programs\Anzoth";
    private const string PackageArchiveName = "codex-package-x86_64-pc-windows-msvc.tar.gz";
    private const string PackageChecksumName = "codex-package_SHA256SUMS";
    private const string UserAgent = "AnzothInstaller/1.0";

    public static int Main(string[] args)
    {
        try
        {
            var options = InstallerOptions.Parse(args);
            if (options.ShowHelp)
            {
                PrintHelp();
                return 0;
            }

            if (options.ShowVersion)
            {
                Console.WriteLine($"Anzoth Setup {GetVersion()}");
                return 0;
            }

            if (options.Uninstall)
            {
                Uninstall(options);
                return 0;
            }

            Install(options);
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Anzoth installer error: {ex.Message}");
            return 1;
        }
    }

    private static void Install(InstallerOptions options)
    {
        var installRoot = ExpandPath(options.InstallRoot ?? DefaultInstallRoot);
        var binDir = Path.Combine(installRoot, "bin");
        using var packageSource = ResolvePackageSource(options);
        var sourceRoot = packageSource.Path;

        Console.WriteLine($"=> Installing Anzoth CLI to {installRoot}");
        if (Directory.Exists(installRoot))
        {
            Console.WriteLine("=> Updating existing installation");
            Directory.Delete(installRoot, recursive: true);
        }

        CopyDirectory(sourceRoot, installRoot);
        EnsureAnzothEntrypoint(binDir);
        UpdateUserPath(binDir);
        VerifyInstalledCommand(Path.Combine(binDir, "anzoth.exe"));

        Console.WriteLine("=> PATH updated for future Command Prompt and PowerShell sessions");
        Console.WriteLine("Anzoth CLI installed successfully.");
    }

    private static void Uninstall(InstallerOptions options)
    {
        var installRoot = ExpandPath(options.InstallRoot ?? DefaultInstallRoot);
        var binDir = Path.Combine(installRoot, "bin");

        Console.WriteLine($"=> Removing Anzoth CLI from {installRoot}");
        RemoveFromUserPath(binDir);

        if (Directory.Exists(installRoot))
        {
            Directory.Delete(installRoot, recursive: true);
        }

        Console.WriteLine("Anzoth CLI removed successfully.");
    }

    private static PackageSource ResolvePackageSource(InstallerOptions options)
    {
        if (!string.IsNullOrWhiteSpace(options.PackageDir))
        {
            var packageDir = Path.GetFullPath(options.PackageDir);
            ValidatePackageDirectory(packageDir);
            Console.WriteLine($"=> Using local package directory {packageDir}");
            return new PackageSource(packageDir, cleanup: false);
        }

        if (!string.IsNullOrWhiteSpace(options.PackageArchive))
        {
            var archivePath = Path.GetFullPath(options.PackageArchive);
            if (!File.Exists(archivePath))
            {
                throw new FileNotFoundException("Package archive not found", archivePath);
            }

            var stagingDir = Path.Combine(Path.GetTempPath(), $"anzoth-installer-{Guid.NewGuid():N}");
            Directory.CreateDirectory(stagingDir);
            ExtractTarGz(archivePath, stagingDir);
            ValidatePackageDirectory(stagingDir);
            Console.WriteLine($"=> Using local package archive {archivePath}");
            return new PackageSource(stagingDir, cleanup: true);
        }

        return DownloadReleasePackage(options.Repository, options.ReleaseVersion);
    }

    private static PackageSource DownloadReleasePackage(string? repository, string? releaseVersion)
    {
        var repo = string.IsNullOrWhiteSpace(repository) ? DefaultRepository : repository;
        var version = string.IsNullOrWhiteSpace(releaseVersion) ? "latest" : releaseVersion;
        var metadataUri = version == "latest"
            ? $"https://api.github.com/repos/{repo}/releases/latest"
            : $"https://api.github.com/repos/{repo}/releases/tags/rust-v{version}";

        using var client = CreateHttpClient();
        using var response = client.GetAsync(metadataUri).GetAwaiter().GetResult();
        response.EnsureSuccessStatusCode();

        using var document = JsonDocument.Parse(response.Content.ReadAsStream());
        var release = document.RootElement;
        var resolvedVersion = version == "latest"
            ? NormalizeVersion(release.GetProperty("tag_name").GetString())
            : version;

        var asset = FindAsset(release, PackageArchiveName);
        var archiveUri = asset.browser_download_url;
        var archiveDigest = ParseDigest(asset.digest);

        var downloadPath = Path.Combine(Path.GetTempPath(), $"anzoth-release-{Guid.NewGuid():N}.tar.gz");
        using (var download = client.GetAsync(archiveUri).GetAwaiter().GetResult())
        {
            download.EnsureSuccessStatusCode();
            using var input = download.Content.ReadAsStream();
            using var output = File.Create(downloadPath);
            input.CopyTo(output);
        }

        VerifySha256(downloadPath, archiveDigest);

        var stagingDir = Path.Combine(Path.GetTempPath(), $"anzoth-release-{Guid.NewGuid():N}");
        Directory.CreateDirectory(stagingDir);
        ExtractTarGz(downloadPath, stagingDir);
        ValidatePackageDirectory(stagingDir);
        Console.WriteLine($"=> Downloaded Anzoth release {resolvedVersion} from {repo}");
        return new PackageSource(stagingDir, cleanup: true);
    }

    private static HttpClient CreateHttpClient()
    {
        var client = new HttpClient();
        client.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue("AnzothInstaller", "1.0"));
        client.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
        return client;
    }

    private static (string browser_download_url, string digest) FindAsset(JsonElement release, string assetName)
    {
        foreach (var asset in release.GetProperty("assets").EnumerateArray())
        {
            var name = asset.GetProperty("name").GetString();
            if (name == assetName)
            {
                var url = asset.GetProperty("browser_download_url").GetString();
                var digest = asset.TryGetProperty("digest", out var digestElement)
                    ? digestElement.GetString()
                    : null;
                if (string.IsNullOrWhiteSpace(url) || string.IsNullOrWhiteSpace(digest))
                {
                    throw new InvalidOperationException($"Missing download metadata for release asset {assetName}.");
                }

                return (url, digest);
            }
        }

        throw new InvalidOperationException($"Release asset not found: {assetName}");
    }

    private static string NormalizeVersion(string? tagName)
    {
        if (string.IsNullOrWhiteSpace(tagName))
        {
            throw new InvalidOperationException("Release metadata is missing tag_name.");
        }

        return tagName.StartsWith("rust-v", StringComparison.OrdinalIgnoreCase)
            ? tagName[6..]
            : tagName;
    }

    private static string ParseDigest(string? digest)
    {
        if (string.IsNullOrWhiteSpace(digest) || !digest.StartsWith("sha256:", StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidOperationException("Release asset digest is missing or invalid.");
        }

        return digest["sha256:".Length..].ToLowerInvariant();
    }

    private static void ValidatePackageDirectory(string packageDir)
    {
        string[] requiredFiles =
        {
            "codex-package.json",
            Path.Combine("bin", "codex.exe"),
            Path.Combine("bin", "codex-code-mode-host.exe"),
            Path.Combine("codex-path", "rg.exe"),
            Path.Combine("codex-resources", "codex-command-runner.exe"),
            Path.Combine("codex-resources", "codex-windows-sandbox-setup.exe"),
        };

        foreach (var requiredFile in requiredFiles)
        {
            if (!File.Exists(Path.Combine(packageDir, requiredFile)))
            {
                throw new FileNotFoundException($"Package file is missing: {requiredFile}", Path.Combine(packageDir, requiredFile));
            }
        }
    }

    private static void CopyDirectory(string source, string destination)
    {
        var sourceFull = Path.GetFullPath(source).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        var destinationFull = Path.GetFullPath(destination).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        Directory.CreateDirectory(destinationFull);

        foreach (var directory in Directory.EnumerateDirectories(sourceFull, "*", SearchOption.AllDirectories))
        {
            var relative = Path.GetRelativePath(sourceFull, directory);
            Directory.CreateDirectory(Path.Combine(destinationFull, relative));
        }

        foreach (var file in Directory.EnumerateFiles(sourceFull, "*", SearchOption.AllDirectories))
        {
            var relative = Path.GetRelativePath(sourceFull, file);
            var targetFile = Path.Combine(destinationFull, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(targetFile)!);
            File.Copy(file, targetFile, overwrite: true);
        }
    }

    private static void EnsureAnzothEntrypoint(string binDir)
    {
        var codexExe = Path.Combine(binDir, "codex.exe");
        var anzothExe = Path.Combine(binDir, "anzoth.exe");
        if (File.Exists(anzothExe))
        {
            return;
        }

        if (!File.Exists(codexExe))
        {
            throw new FileNotFoundException("Installed package is missing codex.exe", codexExe);
        }

        File.Copy(codexExe, anzothExe, overwrite: true);
    }

    private static void VerifyInstalledCommand(string anzothExe)
    {
        var process = Process.Start(new ProcessStartInfo
        {
            FileName = anzothExe,
            Arguments = "--version",
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        });

        if (process is null)
        {
            throw new InvalidOperationException($"Failed to launch installed command: {anzothExe}");
        }

        var output = process.StandardOutput.ReadToEnd();
        var error = process.StandardError.ReadToEnd();
        process.WaitForExit();
        if (process.ExitCode != 0)
        {
            throw new InvalidOperationException(
                $"Installed command verification failed with exit code {process.ExitCode}: {output}{error}");
        }
    }

    private static void UpdateUserPath(string binDir)
    {
        var current = Environment.GetEnvironmentVariable("Path", EnvironmentVariableTarget.User) ?? string.Empty;
        if (PathContains(current, binDir))
        {
            return;
        }

        var updated = string.IsNullOrWhiteSpace(current)
            ? binDir
            : string.Join(';', binDir, current);
        Environment.SetEnvironmentVariable("Path", updated, EnvironmentVariableTarget.User);
        Environment.SetEnvironmentVariable("Path", updated, EnvironmentVariableTarget.Process);
    }

    private static void RemoveFromUserPath(string binDir)
    {
        var current = Environment.GetEnvironmentVariable("Path", EnvironmentVariableTarget.User) ?? string.Empty;
        if (!PathContains(current, binDir))
        {
            return;
        }

        var remaining = current
            .Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Where(entry => !PathsEqual(entry, binDir))
            .ToArray();
        var updated = string.Join(';', remaining);
        Environment.SetEnvironmentVariable("Path", updated, EnvironmentVariableTarget.User);
        Environment.SetEnvironmentVariable("Path", updated, EnvironmentVariableTarget.Process);
    }

    private static bool PathContains(string pathValue, string entry)
        => pathValue
            .Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Any(segment => PathsEqual(segment, entry));

    private static bool PathsEqual(string left, string right)
        => string.Equals(NormalizePathEntry(left), NormalizePathEntry(right), StringComparison.OrdinalIgnoreCase);

    private static string NormalizePathEntry(string path)
        => path.Trim().TrimEnd('\\', '/');

    private static void ExtractTarGz(string archivePath, string destination)
    {
        using var archiveStream = File.OpenRead(archivePath);
        using var gzip = new GZipStream(archiveStream, CompressionMode.Decompress);
        using var reader = new TarReader(gzip);

        TarEntry? entry;
        while ((entry = reader.GetNextEntry()) is not null)
        {
            if (string.IsNullOrWhiteSpace(entry.Name))
            {
                continue;
            }

            var targetPath = Path.GetFullPath(Path.Combine(destination, entry.Name));
            if (!targetPath.StartsWith(Path.GetFullPath(destination).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase)
                && !string.Equals(targetPath, Path.GetFullPath(destination).TrimEnd(Path.DirectorySeparatorChar), StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException($"Archive entry escapes destination: {entry.Name}");
            }

            switch (entry.EntryType)
            {
                case TarEntryType.Directory:
                    Directory.CreateDirectory(targetPath);
                    break;
                case TarEntryType.RegularFile:
                    Directory.CreateDirectory(Path.GetDirectoryName(targetPath)!);
                    using (var file = File.Create(targetPath))
                    {
                        entry.DataStream?.CopyTo(file);
                    }
                    break;
                default:
                    throw new NotSupportedException($"Unsupported tar entry type: {entry.EntryType}");
            }
        }
    }

    private static void VerifySha256(string path, string expectedDigest)
    {
        using var sha256 = SHA256.Create();
        using var stream = File.OpenRead(path);
        var actualDigest = Convert.ToHexString(sha256.ComputeHash(stream)).ToLowerInvariant();
        if (!string.Equals(actualDigest, expectedDigest, StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidOperationException(
                $"Downloaded installer payload checksum mismatch. Expected {expectedDigest} but got {actualDigest}.");
        }
    }

    private static string ExpandPath(string path)
        => Environment.ExpandEnvironmentVariables(path);

    private static string GetVersion()
        => typeof(Program).Assembly.GetName().Version?.ToString() ?? "0.0.0";

    private static void PrintHelp()
    {
        Console.WriteLine("""
Anzoth Setup

Usage:
  Anzoth-Setup-x64.exe [--release <version>] [--repo <owner/repo>] [--package-dir <dir>] [--package-archive <path>]
  Anzoth-Setup-x64.exe --uninstall

Options:
  --release <version>       Install a specific release tag version, or omit for latest.
  --repo <owner/repo>       Release repository to download from. Defaults to lowenmk/anzoth-cli.
  --package-dir <dir>       Install from an unpacked local package directory.
  --package-archive <path>  Install from a local codex-package .tar.gz archive.
  --install-dir <dir>       Override the install root. Defaults to %LOCALAPPDATA%\Programs\Anzoth.
  --uninstall               Remove the install root and PATH entry.
  --version                 Print the installer version.
  --help                    Show this help text.

The installer places Anzoth CLI under %LOCALAPPDATA%\Programs\Anzoth and adds
the bin directory to the current user's PATH.
""");
    }

    private sealed class PackageSource(string path, bool cleanup) : IDisposable
    {
        public string Path { get; } = path;

        public void Dispose()
        {
            if (cleanup && Directory.Exists(Path))
            {
                Directory.Delete(Path, recursive: true);
            }
        }
    }

    private sealed record InstallerOptions(
        string? InstallRoot,
        string? PackageDir,
        string? PackageArchive,
        string? Repository,
        string? ReleaseVersion,
        bool Uninstall,
        bool ShowHelp,
        bool ShowVersion)
    {
        public static InstallerOptions Parse(string[] args)
        {
            string? installRoot = Environment.GetEnvironmentVariable("ANZOTH_INSTALL_DIR");
            string? packageDir = Environment.GetEnvironmentVariable("ANZOTH_PACKAGE_DIR");
            string? packageArchive = Environment.GetEnvironmentVariable("ANZOTH_PACKAGE_ARCHIVE");
            string? repository = Environment.GetEnvironmentVariable("ANZOTH_RELEASE_REPOSITORY") ?? DefaultRepository;
            string? releaseVersion = Environment.GetEnvironmentVariable("ANZOTH_RELEASE");
            bool uninstall = false;
            bool showHelp = false;
            bool showVersion = false;

            for (var i = 0; i < args.Length; i++)
            {
                var arg = args[i];
                switch (arg)
                {
                    case "--help":
                    case "-h":
                        showHelp = true;
                        break;
                    case "--version":
                        showVersion = true;
                        break;
                    case "--uninstall":
                        uninstall = true;
                        break;
                    case "--install-dir":
                        installRoot = RequireValue(args, ref i, arg);
                        break;
                    case "--package-dir":
                        packageDir = RequireValue(args, ref i, arg);
                        break;
                    case "--package-archive":
                        packageArchive = RequireValue(args, ref i, arg);
                        break;
                    case "--repo":
                        repository = RequireValue(args, ref i, arg);
                        break;
                    case "--release":
                        releaseVersion = RequireValue(args, ref i, arg);
                        break;
                    default:
                        throw new ArgumentException($"Unknown argument: {arg}");
                }
            }

            return new InstallerOptions(
                installRoot,
                packageDir,
                packageArchive,
                repository,
                releaseVersion,
                uninstall,
                showHelp,
                showVersion);
        }

        private static string RequireValue(string[] args, ref int index, string optionName)
        {
            if (index + 1 >= args.Length)
            {
                throw new ArgumentException($"{optionName} requires a value.");
            }

            index++;
            return args[index];
        }
    }
}
