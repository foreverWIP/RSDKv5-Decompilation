#if RETRO_USE_MOD_LOADER
// both v3 and v4 use these
std::vector<SceneListEntry> listData;
std::vector<SceneListInfo> listCategory;
#endif

namespace RSDK
{
namespace Legacy
{

#include "v3/RetroEnginev3.cpp"
#include "v4/RetroEnginev4.cpp"

} // namespace Legacy
} // namespace RSDK