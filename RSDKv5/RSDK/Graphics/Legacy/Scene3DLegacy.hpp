
namespace Legacy
{

#define LEGACY_VERTEXBUFFER_SIZE (0x1000)
#define LEGACY_FACEBUFFER_SIZE   (0x400)

enum FaceFlags {
    FACE_FLAG_TEXTURED_3D      = 0,
    FACE_FLAG_TEXTURED_2D      = 1,
    FACE_FLAG_COLORED_3D       = 2,
    FACE_FLAG_COLORED_2D       = 3,
    FACE_FLAG_FADED            = 4,
    FACE_FLAG_TEXTURED_C       = 5,
    FACE_FLAG_TEXTURED_C_BLEND = 6,
    FACE_FLAG_3DSPRITE         = 7
};

enum MatrixTypes {
    MAT_WORLD = 0,
    MAT_VIEW  = 1,
    MAT_TEMP  = 2,
};

struct Vertex {
    int32 x;
    int32 y;
    int32 z;
    int32 u;
    int32 v;
};

struct Face {
    int32 a;
    int32 b;
    int32 c;
    int32 d;
    uint32 color;
    int32 flag;
};

struct DrawListEntry3D {
    int32 faceID;
    int32 depth;
};

extern int32 vertexCount;
extern int32 faceCount;

extern Matrix matFinal;
extern Matrix matWorld;
extern Matrix matView;
extern Matrix matTemp;

extern int32 projectionX;
extern int32 projectionY;

extern ScanEdge scanEdgeBufferU[SCREEN_YSIZE * 2];
extern ScanEdge scanEdgeBufferV[SCREEN_YSIZE * 2];

extern Vertex vertexBuffer[LEGACY_VERTEXBUFFER_SIZE];
extern Vertex vertexBufferT[LEGACY_VERTEXBUFFER_SIZE];

extern Face faceBuffer[LEGACY_FACEBUFFER_SIZE];

extern DrawListEntry3D drawList3D[LEGACY_FACEBUFFER_SIZE];

extern int32 fogColor;
extern int32 fogStrength;

void ProcessScanEdge(Vertex *vertA, Vertex *vertB);
void ProcessScanEdgeUV(Vertex *vertA, Vertex *vertB);
void TransformVertexBuffer();
void TransformVertices(Matrix *matrix, int32 startIndex, int32 endIndex);
void Sort3DDrawList();
void Draw3DScene(int32 spriteSheetID);

} // namespace Legacy