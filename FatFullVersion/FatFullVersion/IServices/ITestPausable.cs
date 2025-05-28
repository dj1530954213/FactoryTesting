using System.Threading.Tasks;

namespace FatFullVersion.IServices
{
    public interface ITestPausable
    {
        Task PauseAsync();
        Task ResumeAsync();
        bool IsPaused { get; }
    }
} 